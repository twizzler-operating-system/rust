use crate::spec::{
    crt_objects, Cc, FramePointer, LinkArgs, LinkOutputKind, LinkerFlavor, Lld, PanicStrategy, TargetOptions, TlsModel
};

pub(crate) fn opts(static_only: bool) -> TargetOptions {
    let mut pre_link_args = LinkArgs::new();
    pre_link_args.insert(LinkerFlavor::Gnu(Cc::Yes, Lld::Yes), vec![]);
    pre_link_args.insert(LinkerFlavor::Gnu(Cc::Yes, Lld::No), vec![]);

    TargetOptions {
        os: "twizzler".into(),
        env: if static_only { "minruntime".into() } else { "".into() },
        linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
        linker: Some("rust-lld".into()),
        executables: true,
        pre_link_args,
        pre_link_objects: crt_objects::new(&[
            (LinkOutputKind::DynamicNoPicExe, &["crti.o", "crtbegin.o"]),
            (LinkOutputKind::DynamicPicExe, &["crti.o", "crtbeginS.o"]),
            (LinkOutputKind::StaticNoPicExe, &["crti.o", "crtbegin.o"]),
            (LinkOutputKind::StaticPicExe, &["crti.o", "crtbeginS.o"]),
        ]),
        post_link_objects: crt_objects::new(&[
            (LinkOutputKind::DynamicNoPicExe, &["crtend.o", "crtn.o"]),
            (LinkOutputKind::DynamicPicExe, &["crtendS.o", "crtn.o"]),
            (LinkOutputKind::StaticNoPicExe, &["crtend.o", "crtn.o"]),
            (LinkOutputKind::StaticPicExe, &["crtendS.o", "crtn.o"]),
        ]),
        panic_strategy: PanicStrategy::Unwind,
        position_independent_executables: !static_only,
        static_position_independent_executables: !static_only,
        tls_model: if static_only { TlsModel::LocalExec } else { TlsModel::GeneralDynamic },
        crt_static_default: static_only,
        crt_static_respected: false,
        crt_static_allows_dylibs: !static_only,
        dynamic_linking: !static_only,
        has_thread_local: true,
        frame_pointer: FramePointer::NonLeaf,
        ..Default::default()
    }
}
