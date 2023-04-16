mod vodafone;
mod bcr;
mod decathlon;
mod samsung;
mod allianz;
mod auchan;   
mod autonom;
mod brd;
mod draexlmaier;
mod enel;
mod eon;
mod fedex;
mod generali;
mod hm;
mod kaufland;
mod linde;
mod medicover;
use tokio::join;

#[tokio::main]
async fn main() {

    let enel_result = enel::scrape();
    let vodafone_future = vodafone::scrape();
    let bcr_future = bcr::scrape();
    let decathlon_result = decathlon::scrape();
    let samsung_result = samsung::scrape();
    let allianz_result = allianz::scrape();
    let auchan_result = auchan::scrape();
    let autonom_result = autonom::scrape();
    let brd_result = brd::scrape(); 
    let draexlmaier_result = draexlmaier::scrape();
    let eon_result = eon::scrape(); 
    let fedex_result = fedex::scrape();
    let generali_result = generali::scrape();
    let hm_result = hm::scrape();
    let kaufland_resut = kaufland::scrape();
    let medicover_result = medicover::scrape();
    let linde_result = linde::scrape();
    let(
    vodafone_result, 
    bcr_result, 
    decathlon_result, 
    samsung_result, 
    allianz_result,
    auchan_result,
    autonom_result,
    brd_result,
    draexlmaier_result,
    enel_result,
    eon_result,
    fedex_result,
    generali_result,
    hm_result,
    kaufland_resut,
    medicover_result,
    linde_result
) = 
    join!(
        vodafone_future,
        bcr_future,
        decathlon_result,
        samsung_result,
        allianz_result,
        auchan_result,
        autonom_result,
        brd_result,
        draexlmaier_result,
        enel_result,
        eon_result,
        fedex_result,
        generali_result,
        hm_result,
        kaufland_resut,
        medicover_result,
        linde_result
    );  



    if let Err(e) = vodafone_result {
        eprintln!("error vodafone: {}", e);
    }
    if let Err(e) = bcr_result {
        eprintln!("error bcr: {}", e);
    }
    if let Err(e) = decathlon_result {
        eprintln!("error decathlon {}", e);
    }
    if let Err(e) = samsung_result{
        eprintln!("error samsung {}", e);
    }

    if let Err(e) = allianz_result {
        eprintln!("error allianz {}", e);
    }

    if let Err(e) = auchan_result {
        eprintln!("error auchan {}", e)
    }
    if let Err(e) = autonom_result {
        eprintln!("error autonom {}", e)
    }
    if let Err(e) = brd_result{
        eprintln!("error brd {}", e)
    }
    if let Err(e) = draexlmaier_result{
        eprintln!("error draexlmaier {}", e)
    }
    if let Err(e) = enel_result{
        eprintln!("error enel {}", e)
    }
    if let Err(e) = eon_result{
        eprintln!("error eon {}", e)
    }
    if let Err(e) = fedex_result{
        eprintln!("error fedex {}", e)
    }
    if let Err(e) = generali_result{
        eprintln!("error generali {}", e)
    }
    if let Err(e) = hm_result{
        eprintln!("error hm {}", e)
    }
    if let Err(e) = kaufland_resut{
        eprintln!("error kaufland {}", e)
    }
    if let Err(e) = medicover_result{
        eprintln!("error medicover {}", e)
    }
   if let Err(e) = linde_result{
       eprintln!("error linde {}", e)
   }
  

}

