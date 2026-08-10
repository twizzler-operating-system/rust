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

    // Every compartment maps each library into its own (text, data) slot pair, so dynlink re-runs a
    // full relocation pass per compartment. Both flags below cut that work at link time.
    //
    // --pack-dyn-relocs=relr: RELR-compress R_*_RELATIVE entries; dynlink handles DT_RELR. Note
    // this is deliberately not `-z pack-relative-relocs`, which additionally emits a
    // GLIBC_ABI_DT_RELR verneed entry -- dynlink implements no symbol versioning at all.
    //
    // -Bsymbolic-non-weak-functions: bind intra-DSO calls to non-weak functions at link time,
    // demoting symbolic relocations to RELATIVE ones that need no lookup. The `non-weak` variant is
    // required, not a nicety: twz-rt publishes weak malloc/free/getenv/fwrite/fprintf/
    // __cxa_finalize/_Zdl* definitions that libc.so and libc++abi.so are meant to interpose, and
    // plain -Bsymbolic-functions would bind those to the local stubs instead.
    //
    // Caveat for later: this does defeat interposition of *non-weak* functions. Nothing in the
    // initrd relies on that today (the only non-weak functions with more than one definition are
    // __twz_rt_upcall_entry / __rust_entry_from_c, which are per-DSO copies of the same naked
    // trampoline). But note __rust_alloc and friends are defined solely by libstd.so right now; if
    // a binary ever supplies a #[global_allocator], libstd's own allocations would keep using
    // libstd's copy rather than binding to the executable's.
    crate::spec::add_link_args(
        &mut pre_link_args,
        LinkerFlavor::Gnu(Cc::No, Lld::No),
        &["--pack-dyn-relocs=relr", "-Bsymbolic-non-weak-functions"],
    );
    crate::spec::add_link_args(
        &mut pre_link_args,
        LinkerFlavor::Gnu(Cc::Yes, Lld::No),
        &["-Wl,--pack-dyn-relocs=relr", "-Wl,-Bsymbolic-non-weak-functions"],
    );

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
