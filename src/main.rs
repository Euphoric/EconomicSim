#[derive(Copy, Clone, Debug)]
struct Tile
{
    population : f64,
    fertility : f64,
    gold : f64,
}

#[derive(Copy, Clone, Debug)]
struct Producer
{
    population : f64,
    gold: f64,

    happiness : f64,

    products : [f64; 2], // 0 food, 1 goods
    products_ratio : [f64; 2],
    product_usage: [f64; 2]
}

fn main() {

    println!("TYPE;ID;A;B;C;D;E;F;G");

    let mut producers = [Producer {population : 20_f64, gold: 1000_f64, products: [0_f64, 0_f64], products_ratio: [0.0, 0.0], product_usage: [1.0, 1.0], happiness: 0.1}; 5];
    let producer_count = producers.len();

    producers[1].products_ratio[0] = 1.1;
    producers[2].products_ratio[1] = 1.05;
    producers[3].products_ratio[0] = 1.05;
    producers[4].products_ratio[1] = 1.015;

    let mut prices : [f64; 2] = [1_f64, 1_f64];

    for _ in 0..200
    {
        let mut total_product = [0_f64, 0_f64];
        let mut total_gold = 0_f64;

        // produce goods
        for x in 0..producer_count {
            let mut producer = &mut producers[x];

            //let n1 = 1.1;
            //let produced_food = ((( producer.population + 1.0_f64).powf(1.0-n1) - 1.0)/(1.0-n1)) * producer.products_ratio[0];
            let produced_food = producer.population * producer.products_ratio[0];
            producer.products[0] += produced_food;

            //let n1 = 1.1;
            //let produced_goods = ((( producer.population + 1.0_f64).powf(1.0-n1) - 1.0)/(1.0-n1)) * producer.products_ratio[1];
            let produced_goods = producer.population * producer.products_ratio[1];
            producer.products[1] += produced_goods;

            println!("PRODUCERS_PRODUCE;{3};{0:.3};{1:.3};{2:.3};{4:.3};{5:.3};{6:.3}", producer.population, producer.products[0], producer.products[1], x, producer.gold, produced_food, produced_goods);
        }

        for _ in 0..4 {
            // calculate totals for trading
            for x in 0..producer_count {
                let producer = &producers[x];

                total_product[0] += producer.products[0];
                total_product[1] += producer.products[1];
                total_gold += producer.gold;
            }

            // trade between producers
            for product_id in 0..2 {

                let average_gold = total_gold / producer_count as f64;
                let needed_resources = total_product[product_id] / producer_count as f64;

                let (_, sellers_buyers) : (_, Vec<&mut Producer>) = producers.iter_mut().partition(|prod| prod.products[product_id] > needed_resources && prod.gold > average_gold);
                let (mut buyers, mut sellers) : (Vec<&mut Producer>, Vec<&mut Producer>) = sellers_buyers.into_iter().partition(|prod| prod.products[product_id] < needed_resources);

                if sellers.len() <= 0 || buyers.len() <= 0 {
                    continue;
                }

                let mut wanted_resources = 0_f64;
                let mut offered_gold = 0_f64;

                for buyer in &buyers {
                    wanted_resources += needed_resources - buyer.products[product_id];
                    offered_gold += buyer.gold;
                }

                let mut wanted_gold = 0_f64;
                let mut offered_resource = 0_f64;
                for seller in &sellers {
                    wanted_gold += average_gold - seller.gold;
                    offered_resource += seller.products[product_id] - needed_resources;
                }

                if offered_resource <= 0.1 { // if there is nothing to sell, no trading
                    continue;
                }

                let price = offered_gold / offered_resource;
                prices[product_id] = price;

                let mut total_bough = 0_f64;

                for buyer in &mut buyers {
                    let can_buy = buyer.gold / price;
                    let want = needed_resources - buyer.products[product_id];
                    let buys = can_buy.min(want);
                    buyer.products[product_id] += buys;
                    buyer.gold -= buys * price;
                    total_bough += buys;
                }

                let sold_ratio = total_bough / offered_resource;

                if sold_ratio > 1.0 {
                    println!("ERROR; sold more than is available");
                }

                for seller in &mut sellers {
                    let offered = seller.products[product_id] - needed_resources;
                    let seller_sold = offered * sold_ratio;

                    seller.products[product_id] -= seller_sold;
                    seller.gold += seller_sold * price;
                }

                println!("TRADING;{0:.10};{1:.10};{2:.10};{3:.10};{4:.10};{5:.10};{6:.10}", product_id, price, wanted_resources, offered_gold, offered_resource, wanted_gold, total_bough);
            }
        }
        println!("PRICES;0;{0:.10};{1:.10};", prices[0], prices[1]);

        // consume goods
        for x in 0..producer_count {
            let mut producer = &mut producers[x];

            println!("PRODUCERS_CONSUME_BEFORE;{3};{0:.3};{1:.3};{2:.3};{4:.3};{5:.3}", producer.population, producer.products[0], producer.products[1], x, producer.gold, producer.happiness);

            let needed_food = producer.population * producer.product_usage[0];
            let eaten_food = producer.products[0].min(needed_food);
            producer.products[0] -= eaten_food;

            let needed_goods = producer.population * producer.product_usage[1];
            let used_goods = producer.products[1].min(needed_goods);
            producer.products[1] -= used_goods;

            let food_ratio = eaten_food / needed_food;
            let goods_ratio = used_goods / needed_goods;

            producer.happiness = food_ratio + food_ratio * goods_ratio * 0.5;

            if x == 0 {
                producer.happiness *= 1.01;
            }

            println!("PRODUCERS_CONSUME_AFTER;{3};{0:.3};{1:.3};{2:.3};{4:.3};{5:.3};{6:.3};{7:.3}", producer.population, producer.products[0], producer.products[1], x, producer.gold, producer.happiness, eaten_food, used_goods);
        }

        // move people between producers
        {
            let mut producers_vec: Vec<&mut Producer> = producers.iter_mut().collect::<Vec<_>>();

            producers_vec.sort_by(|a, b| a.happiness.partial_cmp(&b.happiness).unwrap());

            for from in 0 .. producers_vec.len() {
                for to in from+1 .. producers_vec.len(){

                    let move_people = producers_vec[to].happiness - producers_vec[from].happiness;
                    let move_people = move_people * 0.1;
                    let target_population = 0.1_f64.max(producers_vec[from].population - move_people);
                    let move_people = producers_vec[from].population - target_population;

                    producers_vec[from].population -= move_people;
                    producers_vec[to].population += move_people;

                    println!("POPULATION_MOVE;{0:.3};{1:.3};{2:.3};{3:.3};{4:.3};{5:.3}", from, to, producers_vec[from].population, producers_vec[to].population, target_population, move_people);
                }
            }
        }

        // taxes (duh!)
        {
            let tax_rate = 0.2;

            let mut total_tax = 0.0;
            for x in 1..producer_count {
                let tax = producers[x].gold * tax_rate;
                producers[x].gold -= tax;
                total_tax += tax;
            }

            producers[0].gold += total_tax;
        }

        // sanity checks
        {
            for prod in &producers{
                for x in 0..2 {
                    if prod.products[x] < -0.0000001 {
                        println!("ERROR; product {0} is negative", x)
                    }
                }
                if prod.gold < -0.0000001 {
                    println!("ERROR; gold is negative:{0:.10}", prod.gold)
                }
            }
        }
    }
}