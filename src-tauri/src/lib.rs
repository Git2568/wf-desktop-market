mod analysis_cache;
mod analysis_service;
mod catalog_cache;
mod market;
#[cfg(test)]
mod range_hint_tests;
mod reroll_planner;
mod reroll_pool;
mod reroll_service;
mod reroll_solver;
#[cfg(test)]
mod reroll_tests;
mod riven_analysis;
mod riven_roll_data;
mod riven_valuation;
mod valuation_numeric;
#[cfg(test)]
mod valuation_tests;

use analysis_service::{AnalysisDataStatus, AnalysisService, MarketAnalysisResult};
use catalog_cache::CatalogDataStatus;
use market::{
    CatalogUpdateResult, MarketCatalog, MarketFilters, MarketService, MarketSnapshot, RivenWeapon,
};
use reroll_planner::{RerollOptionsResponse, RerollPlanRequest, RerollPlanResponse};
use reroll_service::{RerollOptionsRequest, RerollService};
use riven_analysis::{AnalysisRequest, RangeHintRequest, RangeHintsResult, StatAnalysisResult};
use tauri::Manager;

#[tauri::command]
async fn riven_reroll_options(
    service: tauri::State<'_, RerollService>,
    request: RerollOptionsRequest,
) -> Result<RerollOptionsResponse, String> {
    service.options(request).await
}
#[tauri::command]
async fn riven_reroll_plan(
    service: tauri::State<'_, RerollService>,
    request: RerollPlanRequest,
) -> Result<RerollPlanResponse, String> {
    service.plan(request).await
}

#[tauri::command]
async fn market_weapons(
    service: tauri::State<'_, MarketService>,
) -> Result<Vec<RivenWeapon>, String> {
    service.weapons().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn market_search(
    service: tauri::State<'_, MarketService>,
    filters: MarketFilters,
) -> Result<MarketSnapshot, String> {
    service.search(filters).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn market_catalog(service: tauri::State<'_, MarketService>) -> Result<MarketCatalog, String> {
    service.catalog().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn market_catalog_status(
    service: tauri::State<'_, MarketService>,
) -> Result<CatalogDataStatus, String> {
    service.catalog_status().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn market_update_catalog(
    service: tauri::State<'_, MarketService>,
    force: Option<bool>,
) -> Result<CatalogUpdateResult, String> {
    service
        .update_catalog(force.unwrap_or(false))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn riven_analyze_stats(
    service: tauri::State<'_, AnalysisService>,
    request: AnalysisRequest,
) -> Result<StatAnalysisResult, String> {
    service.stats(request).await
}

#[tauri::command]
async fn riven_analysis_range_hints(
    service: tauri::State<'_, AnalysisService>,
    request: RangeHintRequest,
) -> Result<RangeHintsResult, String> {
    service.range_hints(request).await
}

#[tauri::command]
async fn riven_analyze_market(
    service: tauri::State<'_, AnalysisService>,
    request: AnalysisRequest,
) -> Result<MarketAnalysisResult, String> {
    service.market(request).await
}

#[tauri::command]
async fn riven_analysis_data_status(
    service: tauri::State<'_, AnalysisService>,
) -> Result<AnalysisDataStatus, String> {
    service.status().await
}

#[tauri::command]
async fn riven_refresh_analysis_data(
    service: tauri::State<'_, AnalysisService>,
    force: Option<bool>,
) -> Result<AnalysisDataStatus, String> {
    service.refresh(force.unwrap_or(false)).await
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let path = app
                .path()
                .app_local_data_dir()?
                .join("market-catalog.sqlite3");
            let market = MarketService::with_catalog_path(path);
            app.manage(RerollService::new(market.clone()));
            let analysis_path = app
                .path()
                .app_local_data_dir()?
                .join("riven-analysis.sqlite3");
            app.manage(AnalysisService::new(analysis_path, market.clone()));
            app.manage(market);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            market_weapons,
            market_catalog,
            market_catalog_status,
            market_update_catalog,
            market_search,
            riven_analyze_stats,
            riven_analysis_range_hints,
            riven_analyze_market,
            riven_analysis_data_status,
            riven_refresh_analysis_data,
            riven_reroll_options,
            riven_reroll_plan
        ])
        .run(tauri::generate_context!())
        .expect("error while running WM Market");
}
