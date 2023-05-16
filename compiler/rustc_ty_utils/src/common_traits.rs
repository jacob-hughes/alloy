//! Queries for checking whether a type implements one of a few common traits.

use rustc_hir::lang_items::LangItem;
use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::query::Providers;
use rustc_middle::ty::{self, Ty, TyCtxt};
use rustc_trait_selection::traits;

fn is_copy_raw<'tcx>(tcx: TyCtxt<'tcx>, query: ty::ParamEnvAnd<'tcx, Ty<'tcx>>) -> bool {
    is_item_raw(tcx, query, LangItem::Copy)
}

fn is_sized_raw<'tcx>(tcx: TyCtxt<'tcx>, query: ty::ParamEnvAnd<'tcx, Ty<'tcx>>) -> bool {
    is_item_raw(tcx, query, LangItem::Sized)
}

fn is_freeze_raw<'tcx>(tcx: TyCtxt<'tcx>, query: ty::ParamEnvAnd<'tcx, Ty<'tcx>>) -> bool {
    is_item_raw(tcx, query, LangItem::Freeze)
}

fn is_unpin_raw<'tcx>(tcx: TyCtxt<'tcx>, query: ty::ParamEnvAnd<'tcx, Ty<'tcx>>) -> bool {
    is_item_raw(tcx, query, LangItem::Unpin)
}

fn is_no_trace_raw<'tcx>(tcx: TyCtxt<'tcx>, query: ty::ParamEnvAnd<'tcx, Ty<'tcx>>) -> bool {
    is_item_raw(tcx, query, LangItem::NoTrace)
}

fn is_conservative_raw<'tcx>(tcx: TyCtxt<'tcx>, query: ty::ParamEnvAnd<'tcx, Ty<'tcx>>) -> bool {
    is_item_raw(tcx, query, LangItem::Conservative)
}

fn is_no_finalize_raw<'tcx>(tcx: TyCtxt<'tcx>, query: ty::ParamEnvAnd<'tcx, Ty<'tcx>>) -> bool {
    is_item_raw(tcx, query, LangItem::NoFinalize)
}

fn is_gc_smart_pointer_raw<'tcx>(
    tcx: TyCtxt<'tcx>,
    query: ty::ParamEnvAnd<'tcx, Ty<'tcx>>,
) -> bool {
    is_item_raw(tcx, query, LangItem::GcSmartPointer)
}

fn is_item_raw<'tcx>(
    tcx: TyCtxt<'tcx>,
    query: ty::ParamEnvAnd<'tcx, Ty<'tcx>>,
    item: LangItem,
) -> bool {
    let (param_env, ty) = query.into_parts();
    let trait_def_id = tcx.require_lang_item(item, None);
    let infcx = tcx.infer_ctxt().build();
    traits::type_known_to_meet_bound_modulo_regions(&infcx, param_env, ty, trait_def_id)
}

pub(crate) fn provide(providers: &mut Providers) {
    *providers = ty::query::Providers {
        is_copy_raw,
        is_sized_raw,
        is_freeze_raw,
        is_unpin_raw,
        is_conservative_raw,
        is_no_trace_raw,
        is_gc_smart_pointer_raw,
        is_no_finalize_raw,
        ..*providers
    };
}
