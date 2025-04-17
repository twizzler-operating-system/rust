use crate::spec::{
    Cc, FramePointer, LinkArgs, LinkOutputKind, LinkSelfContainedDefault, LinkerFlavor, Lld,
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
        os: "twizzler".into(),
        env: "".into(),
        linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
        linker: Some("rust-lld".into()),
        executables: true,
        pre_link_args,
        pre_link_objects_self_contained: crt_objects::new(&[
            (LinkOutputKind::DynamicNoPicExe, &["crti.o", "crtbegin.o"]),
            (LinkOutputKind::DynamicPicExe, &["crti.o", "crtbeginS.o"]),
            (LinkOutputKind::StaticNoPicExe, &["crti.o", "crtbegin.o"]),
            (LinkOutputKind::StaticPicExe, &["crti.o", "crtbeginS.o"]),
        ]),
        post_link_objects_self_contained: crt_objects::new(&[
            (LinkOutputKind::DynamicNoPicExe, &["crtend.o", "crtn.o"]),
            (LinkOutputKind::DynamicPicExe, &["crtendS.o", "crtn.o"]),
            (LinkOutputKind::StaticNoPicExe, &["crtend.o", "crtn.o"]),
            (LinkOutputKind::StaticPicExe, &["crtendS.o", "crtn.o"]),
        ]),
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
        link_self_contained: LinkSelfContainedDefault::True,
        ..Default::default()
    }
}
