// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use rustc::hir;
use rustc::hir::def_id::DefId;
use rustc::hir::itemlikevisit::ItemLikeVisitor;
use rustc::ty::subst::{Kind, Subst, UnpackedKind};
use rustc::ty::{self, Ty, TyCtxt};
use rustc::util::nodemap::FxHashMap;

use super::explicit::ExplicitPredicatesMap;
use super::utils::*;

/// Infer predicates for the items in the crate.
///
/// global_inferred_outlives: this is initially the empty map that
///     was generated by walking the items in the crate. This will
///     now be filled with inferred predicates.
pub fn infer_predicates<'tcx>(
    tcx: TyCtxt<'_, 'tcx, 'tcx>,
    explicit_map: &mut ExplicitPredicatesMap<'tcx>,
) -> FxHashMap<DefId, RequiredPredicates<'tcx>> {
    debug!("infer_predicates");

    let mut predicates_added = true;

    let mut global_inferred_outlives = FxHashMap::default();

    // If new predicates were added then we need to re-calculate
    // all crates since there could be new implied predicates.
    while predicates_added {
        predicates_added = false;

        let mut visitor = InferVisitor {
            tcx: tcx,
            global_inferred_outlives: &mut global_inferred_outlives,
            predicates_added: &mut predicates_added,
            explicit_map: explicit_map,
        };

        // Visit all the crates and infer predicates
        tcx.hir.krate().visit_all_item_likes(&mut visitor);
    }

    global_inferred_outlives
}

pub struct InferVisitor<'cx, 'tcx: 'cx> {
    tcx: TyCtxt<'cx, 'tcx, 'tcx>,
    global_inferred_outlives: &'cx mut FxHashMap<DefId, RequiredPredicates<'tcx>>,
    predicates_added: &'cx mut bool,
    explicit_map: &'cx mut ExplicitPredicatesMap<'tcx>,
}

impl<'cx, 'tcx> ItemLikeVisitor<'tcx> for InferVisitor<'cx, 'tcx> {
    fn visit_item(&mut self, item: &hir::Item) {
        let item_did = self.tcx.hir.local_def_id(item.id);

        debug!("InferVisitor::visit_item(item={:?})", item_did);

        let node_id = self.tcx
            .hir
            .as_local_node_id(item_did)
            .expect("expected local def-id");
        let item = match self.tcx.hir.get(node_id) {
            hir::map::NodeItem(item) => item,
            _ => bug!(),
        };

        let mut item_required_predicates = RequiredPredicates::default();
        match item.node {
            hir::ItemKind::Union(..) | hir::ItemKind::Enum(..) | hir::ItemKind::Struct(..) => {
                let adt_def = self.tcx.adt_def(item_did);

                // Iterate over all fields in item_did
                for field_def in adt_def.all_fields() {
                    // Calculating the predicate requirements necessary
                    // for item_did.
                    //
                    // For field of type &'a T (reference) or Adt
                    // (struct/enum/union) there will be outlive
                    // requirements for adt_def.
                    let field_ty = self.tcx.type_of(field_def.did);
                    insert_required_predicates_to_be_wf(
                        self.tcx,
                        field_ty,
                        self.global_inferred_outlives,
                        &mut item_required_predicates,
                        &mut self.explicit_map,
                    );
                }
            }

            _ => {}
        };

        // If new predicates were added (`local_predicate_map` has more
        // predicates than the `global_inferred_outlives`), the new predicates
        // might result in implied predicates for their parent types.
        // Therefore mark `predicates_added` as true and which will ensure
        // we walk the crates again and re-calculate predicates for all
        // items.
        let item_predicates_len: usize = self.global_inferred_outlives
            .get(&item_did)
            .map(|p| p.len())
            .unwrap_or(0);
        if item_required_predicates.len() > item_predicates_len {
            *self.predicates_added = true;
            self.global_inferred_outlives
                .insert(item_did, item_required_predicates);
        }
    }

    fn visit_trait_item(&mut self, _trait_item: &'tcx hir::TraitItem) {}

    fn visit_impl_item(&mut self, _impl_item: &'tcx hir::ImplItem) {}
}

fn insert_required_predicates_to_be_wf<'tcx>(
    tcx: TyCtxt<'_, 'tcx, 'tcx>,
    field_ty: Ty<'tcx>,
    global_inferred_outlives: &FxHashMap<DefId, RequiredPredicates<'tcx>>,
    required_predicates: &mut RequiredPredicates<'tcx>,
    explicit_map: &mut ExplicitPredicatesMap<'tcx>,
) {
    for ty in field_ty.walk() {
        match ty.sty {
            // The field is of type &'a T which means that we will have
            // a predicate requirement of T: 'a (T outlives 'a).
            //
            // We also want to calculate potential predicates for the T
            ty::Ref(region, rty, _) => {
                debug!("Ref");
                insert_outlives_predicate(tcx, rty.into(), region, required_predicates);
            }

            // For each Adt (struct/enum/union) type `Foo<'a, T>`, we
            // can load the current set of inferred and explicit
            // predicates from `global_inferred_outlives` and filter the
            // ones that are TypeOutlives.
            ty::Adt(def, substs) => {
                // First check the inferred predicates
                //
                // Example 1:
                //
                //     struct Foo<'a, T> {
                //         field1: Bar<'a, T>
                //     }
                //
                //     struct Bar<'b, U> {
                //         field2: &'b U
                //     }
                //
                // Here, when processing the type of `field1`, we would
                // request the set of implicit predicates computed for `Bar`
                // thus far. This will initially come back empty, but in next
                // round we will get `U: 'b`. We then apply the substitution
                // `['b => 'a, U => T]` and thus get the requirement that `T:
                // 'a` holds for `Foo`.
                debug!("Adt");
                if let Some(unsubstituted_predicates) = global_inferred_outlives.get(&def.did) {
                    for unsubstituted_predicate in unsubstituted_predicates {
                        // `unsubstituted_predicate` is `U: 'b` in the
                        // example above.  So apply the substitution to
                        // get `T: 'a` (or `predicate`):
                        let predicate = unsubstituted_predicate.subst(tcx, substs);
                        insert_outlives_predicate(
                            tcx,
                            predicate.0,
                            predicate.1,
                            required_predicates,
                        );
                    }
                }

                // Check if the type has any explicit predicates that need
                // to be added to `required_predicates`
                // let _: () = substs.region_at(0);
                check_explicit_predicates(
                    tcx,
                    &def.did,
                    substs,
                    required_predicates,
                    explicit_map,
                    IgnoreSelfTy(false),
                );
            }

            ty::Dynamic(obj, ..) => {
                // This corresponds to `dyn Trait<..>`. In this case, we should
                // use the explicit predicates as well.

                debug!("Dynamic");
                debug!("field_ty = {}", &field_ty);
                debug!("ty in field = {}", &ty);
                if let Some(ex_trait_ref) = obj.principal() {
                    // Here, we are passing the type `usize` as a
                    // placeholder value with the function
                    // `with_self_ty`, since there is no concrete type
                    // `Self` for a `dyn Trait` at this
                    // stage. Therefore when checking explicit
                    // predicates in `check_explicit_predicates` we
                    // need to ignore checking the explicit_map for
                    // Self type.
                    let substs = ex_trait_ref
                        .with_self_ty(tcx, tcx.types.usize)
                        .skip_binder()
                        .substs;
                    check_explicit_predicates(
                        tcx,
                        &ex_trait_ref.skip_binder().def_id,
                        substs,
                        required_predicates,
                        explicit_map,
                        IgnoreSelfTy(true),
                    );
                }
            }

            ty::Projection(obj) => {
                // This corresponds to `<T as Foo<'a>>::Bar`. In this case, we should use the
                // explicit predicates as well.
                debug!("Projection");
                check_explicit_predicates(
                    tcx,
                    &tcx.associated_item(obj.item_def_id).container.id(),
                    obj.substs,
                    required_predicates,
                    explicit_map,
                    IgnoreSelfTy(false),
                );
            }

            _ => {}
        }
    }
}

pub struct IgnoreSelfTy(bool);

/// We also have to check the explicit predicates
/// declared on the type.
///
///     struct Foo<'a, T> {
///         field1: Bar<T>
///     }
///
///     struct Bar<U> where U: 'static, U: Foo {
///         ...
///     }
///
/// Here, we should fetch the explicit predicates, which
/// will give us `U: 'static` and `U: Foo`. The latter we
/// can ignore, but we will want to process `U: 'static`,
/// applying the substitution as above.
pub fn check_explicit_predicates<'tcx>(
    tcx: TyCtxt<'_, 'tcx, 'tcx>,
    def_id: &DefId,
    substs: &[Kind<'tcx>],
    required_predicates: &mut RequiredPredicates<'tcx>,
    explicit_map: &mut ExplicitPredicatesMap<'tcx>,
    ignore_self_ty: IgnoreSelfTy,
) {
    debug!("def_id = {:?}", &def_id);
    debug!("substs = {:?}", &substs);
    debug!("explicit_map =  {:?}", explicit_map);
    debug!("required_predicates = {:?}", required_predicates);
    let explicit_predicates = explicit_map.explicit_predicates_of(tcx, *def_id);

    for outlives_predicate in explicit_predicates.iter() {
        debug!("outlives_predicate = {:?}", &outlives_predicate);

        // Careful: If we are inferring the effects of a `dyn Trait<..>`
        // type, then when we look up the predicates for `Trait`,
        // we may find some that reference `Self`. e.g., perhaps the
        // definition of `Trait` was:
        //
        // ```
        // trait Trait<'a, T> where Self: 'a  { .. }
        // ```
        //
        // we want to ignore such predicates here, because
        // there is no type parameter for them to affect. Consider
        // a struct containing `dyn Trait`:
        //
        // ```
        // struct MyStruct<'x, X> { field: Box<dyn Trait<'x, X>> }
        // ```
        //
        // The `where Self: 'a` predicate refers to the *existential, hidden type*
        // that is represented by the `dyn Trait`, not to the `X` type parameter
        // (or any other generic parameter) declared on `MyStruct`.
        //
        // Note that we do this check for self **before** applying `substs`. In the
        // case that `substs` come from a `dyn Trait` type, our caller will have
        // included `Self = dyn Trait<'x, X>` as the value for `Self`. If we were
        // to apply the substs, and not filter this predicate, we might then falsely
        // conclude that e.g. `X: 'x` was a reasonable inferred requirement.
        if let UnpackedKind::Type(ty) = outlives_predicate.0.unpack() {
            if ty.is_self() && ignore_self_ty.0 {
                debug!("skipping self ty = {:?}", &ty);
                continue;
            }
        }

        let predicate = outlives_predicate.subst(tcx, substs);
        debug!("predicate = {:?}", &predicate);
        insert_outlives_predicate(tcx, predicate.0.into(), predicate.1, required_predicates);
    }
}
