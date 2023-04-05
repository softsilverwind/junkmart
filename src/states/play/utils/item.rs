use bevy::utils::HashSet;

use rand::{prelude::*, seq::SliceRandom};

use super::{
    SideEffect,
    super::{
        resources::Money,
        utils::StatusEffect
    }
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Item
{
    Barrel,
    Burger,
    Gun,
    Pill,
    Screwdriver
}

fn random_money(base: i32) -> Money
{
    Money::new(base + thread_rng().gen_range(-base / 20..=base / 20))
}

impl Item
{
    pub fn new_random(turn: i32, prev_item: Option<Item>) -> Self
    {
        use Item::*;
        let items: &[Item] = match turn {
            1..=5 => &[Burger, Screwdriver],
            6..=20 => &[Burger, Screwdriver, Gun, Pill],
            21 => &[Barrel],
            _ => &[Burger, Screwdriver, Gun, Pill]
        };

        let mut set: HashSet<Item> = items.into_iter().copied().collect();

        if let Some(item) = prev_item {
            set.remove(&item);
        }

        set.into_iter().choose(&mut thread_rng()).unwrap()
    }

    pub fn request(&self) -> &'static str
    {
        use Item::*;
        let words: &[&'static str] = match self {
            Barrel => &["a radioactive barrel", "a barrel with radioactive goo"],
            Burger => &["a shipment of burgers", "food", "edibles"],
            Gun => &["guns", "pieces that go bang bang", "weapons"],
            Pill => &["a pill", "pills", "drugs"],
            Screwdriver => &["a screwdriver", "screwdrivers"]
        };

        words.choose(&mut thread_rng()).unwrap()
    }

    pub fn found(&self) -> &'static str
    {
        use Item::*;
        let words: &[&'static str] = match self {
            Barrel => &["radioactive barrels", "barrels with radioactive goo", "barrels with the nuclear trefoil sign"],
            Burger => &["burgers", "junk food", "food"],
            Gun => &["guns", "pistols", "firearms"],
            Pill => &["pills", "drugs", "medicine"],
            Screwdriver => &["screwdrivers"]
        };

        words.choose(&mut thread_rng()).unwrap()
    }

    pub fn gain(&self) -> Money
    {
        use Item::*;
        let base = match self {
            Barrel => 8_000_000,
            Burger => 10,
            Gun => 300,
            Pill => 400,
            Screwdriver => 10
        };

        random_money(base)
    }

    pub fn side_effect(&self) -> (String, SideEffect)
    {
        use Item::*;
        use StatusEffect::*;
        use SideEffect::*;

        let mut rng = thread_rng();

        match self {
            Barrel => match rng.gen_range(0..5) {
                0 => ("".to_string(), ToggleCancer),
                1 => {
                    let money = random_money(200);
                    (format!("You got scared and had to eat all the iodine tablets, didn't you? Restocking cost you {money}!"), MoneyLoss(money))
                },
                2 => ("The government raided the junkyard to find any more runaway radioactives! They sure left a mess and moved everything around!".to_string(), Reshuffle),
                3 => {
                    let money = random_money(2000);
                    (format!("You had to go to the ER with severe radiation positioning. You are ok now, but the bill was {money}!"), MoneyLoss(money))
                },
                4 => ("The radioactive goo spilled and made a mess! Luckily the customer helped you clean up, before promptly dying from radiation poisoning.".to_string(), CustomerKill),
                _ => panic!()
            },
            Burger => match rng.gen_range(0..4) {
                0 => (r#"You asked yourself, "what could go wrong" and ate the burger. That was when you felt your stomach slowly turning upside down."#.to_string(), StatusEffectEnable(Diarrhea, 3)),
                1 => {
                    let money = random_money(500);
                    (format!("Clearly, a bite won't hurt? After a severe food poisoning, the hospital thinks otherwise. Your idiocy cost {money}."), MoneyLoss(money))
                },
                2 => (r#""Just a small bite," you muttered, "it won't hurt". Then you ran to the bathroom to puke. The customer got angry waiting and left."#.to_string(), CustomerKill),
                3 => ("Mmm, tasty!".to_string(), NoEffect),
                _ => panic!()
            },
            Gun => match rng.gen_range(0..4) {
                0 => {
                    let money = random_money(1000);
                    (format!("You accidentally shot yourself in the foot! An ambulance is on the way! Better have the {money} in hand!"), MoneyLoss(money))
                },
                1 => ("The bullet flew across the junkyard, ricocheting on walls, chests and the stop sign, finally arriving at the customers head.".to_string(), CustomerKill),
                2 => {
                    let money = random_money(250);
                    (format!("The illegal firearm discharge was reported to the police, the fine is {money}!"), MoneyLoss(money))
                }
                3 => {
                    ("The bullet flew across the junkyard, ricocheting on walls, chests and the stop sign, finally exiting the building through the window. Let's hope nobody saw that.".to_string(), NoEffect)
                }
                _ => panic!()
            },
            Pill => match rng.gen_range(0..4) {
                0 => (r#""Mmm, a random pill!", you thought before eating it. Suddenly, your vision became funny."#.to_string(), StatusEffectEnable(Purple, 5)),
                1 => (r#"After eating the pill, a sudden burst of energy ran through your body! "Must reorganize everything!" you cried, as you changed the position of all boxes!"#.to_string(), Reshuffle),
                2 => ("You know the taste of this pill alright. It is Imodium!".to_string(), CureDiarrhea),
                3 => {
                    let money = random_money(1500);
                    (format!("An inspector saw you holding this illegal drug. You paid him {money}. Was it a fine or a bribe? Was he a real inspector? Who knows."), MoneyLoss(money))
                },
                _ => panic!()
            }
            Screwdriver => match rng.gen_range(0..4) {
                0 => {
                    let money = random_money(100);
                    (format!("You got hurt with this rusty screwdriver and must get a tetanus shot! Have {money} at the ready!"), MoneyLoss(money))
                },
                1 => ("As this wasn't what you were searching for, you threw it behind you. The scream of the customer confirmed that the hit was fatal.".to_string(), CustomerKill),
                2 => {
                    let money = random_money(1000);
                    (
                        format!("As this wasn't what you were searching for, you threw it behind you. The scream of the customer confirmed that the hit was not fatal; You got sued for {money} instead."),
                        MoneyLoss(money)
                    )
                },
                3 => {
                    (r#""I have a great idea!" you muttered as you stuck the screwdriver in a power outlet. The electrocution stopped abruptly as the neighborhood transformer exploded. Power will be out for a while, who knows why..."#.to_string(), StatusEffectEnable(LightsOut, 5))
                }
                _ => panic!()
            }
        }

    }

    pub fn global_side_effect(&self) -> Option<&'static str>
    {
        use Item::*;

        let choices: &[&'static str] = match self {
            Barrel => &[
            ],
            Burger => &[
                "A food poisoning epidemic ravages restaurant that feeds the homeless, junk food is to blame.",
                "A person died from a rare strain of streptococcus found in a burger.",
                "Food inspection closes luxurious restaurant after maggots found in burgers."
            ],
            Gun => &[
                "Ten killed in the largest mass shooting our small town has ever seen.",
                "Armed burglars take all valuables from jewellery store, see more on page 10.",
                "Illegal hunting skyrockets as hunters with revoked licenses find new source of illegal guns."
            ],
            Pill => &[
                "Horde of addicts storm police station, fatalities at five and still counting!",
                "CEO of large company overdosed on unidentified drugs, read page 5 for more!",
                "Mysterious drug completely cures patient from both cancer and AIDS, scientists baffled, drug is impossible to recreate!"
            ],
            Screwdriver => &[
                "Rapist with screwdriver shot and killed before he could commit more atrocities!",
                "Gun that shoots screwdrivers is the murder weapon, experts say!",
                "A Rube-Goldberg machine created entirely of screwdrivers falls on the head of the police chief, patrols are doubled!"
            ],
        };

        choices.choose(&mut thread_rng()).copied()
    }
}
