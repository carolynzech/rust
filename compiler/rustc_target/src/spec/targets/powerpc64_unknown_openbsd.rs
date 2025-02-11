use rustc_abi::Endian;

use crate::spec::{Cc, LinkerFlavor, Lld, StackProbeType, Target, TargetOptions, base};

pub(crate) fn target() -> Target {
    let mut base = base::openbsd::opts();
    base.cpu = "ppc64".into();
    base.add_pre_link_args(LinkerFlavor::Gnu(Cc::Yes, Lld::No), &["-m64"]);
    base.max_atomic_width = Some(64);
    base.stack_probes = StackProbeType::Inline;

    Target {
        llvm_target: "powerpc64-unknown-openbsd".into(),
        metadata: crate::spec::TargetMetadata {
            description: Some("OpenBSD/powerpc64".into()),
            tier: Some(3),
            host_tools: Some(true),
            std: Some(true),
        },
        pointer_width: 64,
        data_layout: "E-m:e-Fn32-i64:64-i128:128-n32:64".into(),
        arch: "powerpc64".into(),
        options: TargetOptions { endian: Endian::Big, mcount: "_mcount".into(), ..base },
    }
}
