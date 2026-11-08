use crate::{
    market::MarketService,
    reroll_planner::{
        self, InputAttribute, Planner, RerollOptionsResponse, RerollPlanRequest, RerollPlanResponse,
    },
};
use serde::Deserialize;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use tokio::sync::Mutex as AsyncMutex;
#[derive(Clone, Debug, Deserialize)]
pub struct RerollOptionsRequest {
    pub weapon_slug: String,
    #[serde(default)]
    pub current_attributes: Vec<InputAttribute>,
}
#[derive(Clone)]
pub struct RerollService {
    market: MarketService,
    planner: Arc<Mutex<Planner>>,
    last: Arc<AtomicU64>,
    gate: Arc<AsyncMutex<()>>,
}
impl RerollService {
    pub fn new(market: MarketService) -> Self {
        Self {
            market,
            planner: Arc::new(Mutex::new(Planner::new())),
            last: Arc::new(AtomicU64::new(0)),
            gate: Arc::new(AsyncMutex::new(())),
        }
    }
    pub async fn options(
        &self,
        req: RerollOptionsRequest,
    ) -> Result<RerollOptionsResponse, String> {
        let _g = self.gate.lock().await;
        let c = self.market.catalog().await.map_err(|e| e.to_string())?;
        reroll_planner::options(&c, &req.weapon_slug, &req.current_attributes)
    }
    pub async fn plan(&self, req: RerollPlanRequest) -> Result<RerollPlanResponse, String> {
        let id = self.last.fetch_add(1, Ordering::Relaxed) + 1;
        let _g = self.gate.lock().await;
        let c = self.market.catalog().await.map_err(|e| e.to_string())?;
        let planner = self.planner.clone();
        let last = self.last.clone();
        tokio::task::spawn_blocking(move || {
            if last.load(Ordering::Relaxed) != id {
                return Err("请求已过期".into());
            }
            let mut p = planner
                .lock()
                .map_err(|_| "洗卡 planner 状态锁失败".to_string())?;
            Ok(p.plan(&c, req))
        })
        .await
        .map_err(|e| e.to_string())?
    }
}
