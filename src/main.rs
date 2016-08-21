use std::fs::File;
use std::io::Write;

#[derive(Copy, Clone, Debug)]
struct Tile
{
    population : f64,
    fertility : f64,
    gold : f64,
}

#[derive(Debug)]
struct Producer
{
    name : String,
    population : f64,
    gold: f64,

    happiness : f64,

    products : [f64; 2], // 0 food, 1 goods
    product_usage: [f64; 2],

    production : Production,
}

#[derive(Debug)]
enum Production {
    None,
    Relative {product : usize, ratio : f64},
    Dimmishing {product : usize, multiple : f64, dimminish : f64}
}

impl Production {
    fn produce(&self, population: f64) -> (usize, f64) {
        match self {
            &Production::None => (0, 0.0),
            &Production::Relative { product, ratio } => (product, population * ratio),
            &Production::Dimmishing { product, multiple, dimminish } => (product, ((population+1.0).powf(1.0-dimminish) - 1.0)/(1.0-dimminish) * multiple),
        }
    }
}

// calculates diminishing return coeficient so that 0 population produces zero and at_one population produces 1
fn calc_diminish(product: usize, zero:f64, at_one:f64) -> Production {
    let mult = zero;
    let dimm = zero.log10()/(at_one + 1.0).log10();

    Production::Dimmishing {product:product, multiple:mult, dimminish : dimm }
}

fn main() {

    let mut producers : Vec<Producer> = vec!();

    producers.push(Producer {name:"owners".to_string(),     population : 20_f64, gold: 1000_f64, products: [0_f64, 0_f64], production: Production::None, product_usage: [1.0, 1.0], happiness: 0.0});
    producers.push(Producer {name:"food high".to_string(),  population : 20_f64, gold: 1000_f64, products: [0_f64, 0_f64], production: calc_diminish(0, 2.0, 30.0), product_usage: [1.0, 1.0], happiness: 0.0});
    producers.push(Producer {name:"food low".to_string(),   population : 20_f64, gold: 1000_f64, products: [0_f64, 0_f64], production: calc_diminish(0, 1.5, 60.0), product_usage: [1.0, 1.0], happiness: 0.0});
    producers.push(Producer {name:"goods high".to_string(), population : 20_f64, gold: 1000_f64, products: [0_f64, 0_f64], production: calc_diminish(1, 3.0, 10.0), product_usage: [1.0, 1.0], happiness: 0.0});
    producers.push(Producer {name:"goods low".to_string(),  population : 20_f64, gold: 1000_f64, products: [0_f64, 0_f64], production: calc_diminish(1, 1.1, 20.0), product_usage: [1.0, 1.0], happiness: 0.0});

    let producer_count = producers.len();

    let mut prices : [f64; 2] = [1_f64, 1_f64];

    let mut prices_output : File = File::create("prices.csv").unwrap();
    writeln!(&mut prices_output, "Step;Price 1;Price 2").unwrap();

    let mut population_output : File = File::create("population.csv").unwrap();
    {
        let producers_names = producers.iter().map(|prod| prod.name.to_string()).collect::<Vec<_>>().join(";");
        writeln!(&mut population_output, "Step;{0}", producers_names).unwrap();
    }

    for step_id in 0..500
    {
        let mut total_product = [0_f64, 0_f64];
        let mut total_gold = 0_f64;

        // produce goods
        for x in 0..producer_count {
            let mut producer = &mut producers[x];

            let (item, amount) = producer.production.produce(producer.population);
            producer.products[item] += amount;
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

                if sold_ratio > 1.000001 {
                    println!("ERROR; sold more than is available;{0};{1}", offered_resource, total_bough);
                }

                for seller in &mut sellers {
                    let offered = seller.products[product_id] - needed_resources;
                    let seller_sold = offered * sold_ratio;

                    seller.products[product_id] -= seller_sold;
                    seller.gold += seller_sold * price;
                }
            }
        }

        // consume goods
        for x in 0..producer_count {
            let mut producer = &mut producers[x];

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

        {
            let producers_population = producers.iter().map(|prod| prod.population.to_string()).collect::<Vec<_>>().join(";");
            writeln!(&mut population_output, "{0};{1}", step_id, producers_population).unwrap();
            writeln!(&mut prices_output, "{0};{1:.3};{2:.3}", step_id, prices[0], prices[1]).unwrap();
        }
    }
}
