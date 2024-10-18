use crate::spec::{Cc, LinkerFlavor, Lld, StackProbeType, Target, TargetMetadata};

const LINKER_SCRIPT: &str = include_str!("./x86_64_unknown_twizzler_linker_script.ld");

pub(crate) fn target() -> Target {
    let mut base = crate::spec::base::twizzler::opts(false);
    base.cpu = "x86-64".into();
    base.max_atomic_width = Some(64);
    base.features = "+rdrnd,+rdseed".into();
    base.stack_probes = StackProbeType::Inline;
    base.link_script = Some(LINKER_SCRIPT.into());
    base.pre_link_args
        .get_mut(&LinkerFlavor::Gnu(Cc::Yes, Lld::Yes))
        .unwrap()
        .push("--target=x86_64-unknown-twizzler".into());
    base.pre_link_args
        .get_mut(&LinkerFlavor::Gnu(Cc::Yes, Lld::No))
        .unwrap()
        .push("--target=x86_64-unknown-twizzler".into());
    base.plt_by_default = false;
    
    Target {
        llvm_target: "x86_64-unknown-twizzler".into(),
        pointer_width: 64,
        data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
            .into(),
        arch: "x86_64".into(),
        options: base,
        metadata: TargetMetadata {
            description: Some(std::borrow::Cow::Borrowed("x86_64 Twizzler")),
            tier: None,
            host_tools: None,
            std: Some(true),
        },
    }
}
