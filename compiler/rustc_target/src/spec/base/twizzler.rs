use crate::spec::{
    Cc, Env, FramePointer, LinkArgs, LinkSelfContainedDefault, LinkerFlavor, Lld, Os,
    PanicStrategy, TargetOptions, TlsModel, crt_objects,
};

pub(crate) fn opts() -> TargetOptions {
    let mut pre_link_args = LinkArgs::new();
    let mut post_link_args = LinkArgs::new();
    pre_link_args.insert(LinkerFlavor::Gnu(Cc::Yes, Lld::Yes), vec![]);
    pre_link_args.insert(LinkerFlavor::Gnu(Cc::Yes, Lld::No), vec![]);
    pre_link_args.insert(LinkerFlavor::Gnu(Cc::No, Lld::Yes), vec![]);
    post_link_args.insert(LinkerFlavor::Gnu(Cc::No, Lld::Yes), vec![]);

    TargetOptions {
        os: Os::Twizzler,
        env: Env::Unspecified,
        linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
        linker: Some("ld.lld".into()),
        executables: true,
        pre_link_args,
        pre_link_objects: crt_objects::pre_twizzler_self_contained(),
        post_link_objects: crt_objects::post_twizzler_self_contained(),
        post_link_args,
        panic_strategy: PanicStrategy::Unwind,
        position_independent_executables: true,
        static_position_independent_executables: true,
        tls_model: TlsModel::GeneralDynamic,
        crt_static_default: false,
        crt_static_respected: true,
        crt_static_allows_dylibs: true,
        dynamic_linking: true,
        has_thread_local: true,
        frame_pointer: FramePointer::NonLeaf,
        link_self_contained: LinkSelfContainedDefault::False,
        ..Default::default()
    }
}
