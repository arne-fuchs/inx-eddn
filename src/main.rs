use std::sync::mpsc::channel;
use std::sync::Arc;
use std::{env, thread};
use std::time::Duration;

use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use json::JsonValue;
use crate::edcas_contract::{BodyProperties, PlanetProperties, StarProperties};
use crate::eddn_adapter::EddnAdapter;
use crate::SendError::{NonRepeatableError, RepeatableError};

mod eddn_adapter;
mod edcas_contract;

//abigen!(EDCAS_Contract, "./contracts/EDCAS.abi");

#[tokio::main]
async fn main() {
    //Abigen::new("EDCAS", "./contracts/EDCAS.abi").unwrap().generate().unwrap().write_to_file("./src/edcas_contract.rs").unwrap();
    let args: Vec<String> = env::args().collect();

    let length = args.len();
    for i in 1..length {
        match args[i].as_str() {
            "-evm" => {
                env::set_var("EVM_URL",args[i + 1].clone());
            }
            "-pk" => {
                env::set_var("PRIVATE_KEY",args[i + 1].clone());
            }
            "-sc" => {
                env::set_var("SC_ADDRESS",args[i + 1].clone());
            }
            "-rt" => {
                env::set_var("RETRY_TIMEOUT",args[i + 1].clone());
            }
            "-dt" => {
                env::set_var("DURATION_TIMEOUT",args[i + 1].clone());
            }
            &_ => {}
        }
    }
    
    let (bus_writer, bus_reader) = channel::<String>();
    let eddn = EddnAdapter { bus_writer };
    thread::spawn(move || loop {
        match bus_reader.recv() {
            Ok(string) => match json::parse(string.as_str()) {
                Ok(json) => {
                    interpret_event(json["message"].clone());
                }
                Err(error) => {
                    eprintln!("Error parsing json: {}", error);
                }
            },
            Err(err) => {
                eprintln!("{}", err);
            }
        }
    });
    println!("Ready!");
    eddn.subscribe_to_eddn().await;
}

fn interpret_event(json: JsonValue) {
    match json["event"].to_string().as_str() {
        //Navigation
        //{ "timestamp":"2022-10-16T20:54:45Z",
        // "event":"Location",
        // "DistFromStarLS":1007.705243,
        // "Docked":true,
        // "StationName":"Q2K-BHB",
        // "StationType":"FleetCarrier",
        // "MarketID":3704402432,
        // "StationFaction":{ "Name":"FleetCarrier" },
        // "StationGovernment":"$government_Carrier;",
        // "StationGovernment_Localised":"Privateigentum", "StationServices":[ "dock", "autodock", "commodities", "contacts", "exploration", "outfitting", "crewlounge", "rearm", "refuel", "repair", "shipyard", "engineer", "flightcontroller", "stationoperations", "stationMenu", "carriermanagement", "carrierfuel", "livery", "voucherredemption", "socialspace", "bartender", "vistagenomics" ], "StationEconomy":"$economy_Carrier;", "StationEconomy_Localised":"Privatunternehmen", "StationEconomies":[ { "Name":"$economy_Carrier;", "Name_Localised":"Privatunternehmen", "Proportion":1.000000 } ], "Taxi":false, "Multicrew":false, "StarSystem":"Colonia", "SystemAddress":3238296097059, "StarPos":[-9530.50000,-910.28125,19808.12500], "SystemAllegiance":"Independent", "SystemEconomy":"$economy_Tourism;", "SystemEconomy_Localised":"Tourismus", "SystemSecondEconomy":"$economy_HighTech;", "SystemSecondEconomy_Localised":"Hightech", "SystemGovernment":"$government_Cooperative;", "SystemGovernment_Localised":"Kooperative", "SystemSecurity":"$SYSTEM_SECURITY_low;", "SystemSecurity_Localised":"Geringe Sicherheit", "Population":583869, "Body":"Colonia 2 c", "BodyID":18, "BodyType":"Planet", "Factions":[ { "Name":"Jaques", "FactionState":"Investment", "Government":"Cooperative", "Influence":0.454092, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand1;", "Happiness_Localised":"In Hochstimmung", "MyReputation":100.000000, "RecoveringStates":[ { "State":"PublicHoliday", "Trend":0 } ], "ActiveStates":[ { "State":"Investment" }, { "State":"CivilLiberty" } ] }, { "Name":"Colonia Council", "FactionState":"Boom", "Government":"Cooperative", "Influence":0.331337, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":100.000000, "ActiveStates":[ { "State":"Boom" } ] }, { "Name":"People of Colonia", "FactionState":"None", "Government":"Cooperative", "Influence":0.090818, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":27.956400 }, { "Name":"Holloway Bioscience Institute", "FactionState":"None", "Government":"Corporate", "Influence":0.123752, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":-9.420000, "RecoveringStates":[ { "State":"PirateAttack", "Trend":0 } ] } ], "SystemFaction":{ "Name":"Jaques", "FactionState":"Investment" } }
        "Location" => {}
        //{ "timestamp":"2022-10-16T23:25:31Z",
        // "event":"FSDJump",
        // "Taxi":false,
        // "Multicrew":false,
        // "StarSystem":"Ogmar",
        // "SystemAddress":84180519395914,
        // "StarPos":[-9534.00000,-905.28125,19802.03125],
        // "SystemAllegiance":"Independent",
        // "SystemEconomy":"$economy_HighTech;",
        // "SystemEconomy_Localised":"Hightech",
        // "SystemSecondEconomy":"$economy_Military;",
        // "SystemSecondEconomy_Localised":"Militär",
        // "SystemGovernment":"$government_Confederacy;",
        // "SystemGovernment_Localised":"Konföderation",
        // "SystemSecurity":"$SYSTEM_SECURITY_medium;",
        // "SystemSecurity_Localised":"Mittlere Sicherheit",
        // "Population":151752,
        // "Body":"Ogmar A",
        // "BodyID":1,
        // "BodyType":"Star",
        // "JumpDist":8.625,
        // "FuelUsed":0.024493,
        // "FuelLevel":31.975506,
        // "Factions":[ { "Name":"Jaques", "FactionState":"Election", "Government":"Cooperative", "Influence":0.138384, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand1;", "Happiness_Localised":"In Hochstimmung", "MyReputation":100.000000, "PendingStates":[ { "State":"Outbreak", "Trend":0 } ], "ActiveStates":[ { "State":"Election" } ] }, { "Name":"ICU Colonial Corps", "FactionState":"War", "Government":"Communism", "Influence":0.119192, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":96.402496, "PendingStates":[ { "State":"Expansion", "Trend":0 } ], "ActiveStates":[ { "State":"War" } ] }, { "Name":"Societas Eruditorum de Civitas Dei", "FactionState":"War", "Government":"Dictatorship", "Influence":0.119192, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":46.414799, "ActiveStates":[ { "State":"War" } ] }, { "Name":"GalCop Colonial Defence Commission", "FactionState":"Boom", "Government":"Confederacy", "Influence":0.406061, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":-75.000000, "ActiveStates":[ { "State":"Boom" } ] }, { "Name":"Likedeeler of Colonia", "FactionState":"None", "Government":"Democracy", "Influence":0.068687, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":4.002500 }, { "Name":"Colonia Tech Combine", "FactionState":"Election", "Government":"Cooperative", "Influence":0.138384, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":4.850000, "ActiveStates":[ { "State":"Election" } ] }, { "Name":"Milanov's Reavers", "FactionState":"Bust", "Government":"Anarchy", "Influence":0.010101, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":0.000000, "RecoveringStates":[ { "State":"Terrorism", "Trend":0 } ], "ActiveStates":[ { "State":"Bust" } ] } ], "SystemFaction":{ "Name":"GalCop Colonial Defence Commission", "FactionState":"Boom" }, "Conflicts":[ { "WarType":"election", "Status":"active", "Faction1":{ "Name":"Jaques", "Stake":"Guerrero Military Base", "WonDays":1 }, "Faction2":{ "Name":"Colonia Tech Combine", "Stake":"", "WonDays":0 } }, { "WarType":"war", "Status":"active", "Faction1":{ "Name":"ICU Colonial Corps", "Stake":"Boulaid Command Facility", "WonDays":1 }, "Faction2":{ "Name":"Societas Eruditorum de Civitas Dei", "Stake":"Chatterjee's Respite", "WonDays":0 } } ] }
        "FSDJump" | "CarrierJump" => {
            //TODO Learn tokio
            thread::spawn(||{
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async move {
                        let contract = get_contract().await;
                        let function_call: FunctionCall<
                            Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
                            SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                            (),
                        > = contract.register_system(
                            json["SystemAddress"].as_u64().unwrap(),
                            json["StarSystem"].to_string(),
                            json["SystemAllegiance"].to_string(),
                            json["SystemEconomy"].to_string(),
                            json["SystemSecondEconomy"].to_string(),
                            json["SystemSecurity"].to_string(),
                            json["Population"].as_u64().unwrap_or(0)
                        );
                        //execute_send(function_call).await;
                        execute_send_repeatable(function_call).await;
                    });
            });
        }
        "SupercruiseEntry" => {}
        "SupercruiseExit" => {}
        //{ "timestamp":"2022-10-16T23:25:05Z", "event":"StartJump", "JumpType":"Hyperspace", "StarSystem":"Ogmar", "SystemAddress":84180519395914, "StarClass":"K" }
        "StartJump" => {} //If jump has been initialised
        //{ "timestamp":"2022-10-16T23:24:46Z", "event":"FSDTarget", "Name":"Ogmar", "SystemAddress":84180519395914, "StarClass":"K", "RemainingJumpsInRoute":1 }
        "FSDTarget" => {}     //If system has been targeted
        "NavRoute" => {}      //If route has been set -> check json for further information
        "NavRouteClear" => {} //If navigation is complete -> no further information

        //Approaching
        "ApproachSettlement" => {}
        "ApproachBody" => {}
        "LeaveBody" => {}
        "Liftoff" => {}
        "Touchdown" => {}
        "Embark" => {}
        "Disembark" => {}

        //Scanning
        "DiscoveryScan" => {}
        "FSSAllBodiesFound" => {}
        //{ "timestamp":"2022-10-16T23:46:48Z", "event":"FSSDiscoveryScan", "Progress":0.680273, "BodyCount":21, "NonBodyCount":80, "SystemName":"Ogmar", "SystemAddress":84180519395914 }
        "FSSDiscoveryScan" => {} //Honk
        //{ "timestamp":"2022-07-07T20:58:06Z", "event":"SAASignalsFound", "BodyName":"IC 2391 Sector YE-A d103 B 1", "SystemAddress":3549631072611, "BodyID":15, "Signals":[ { "Type":"$SAA_SignalType_Guardian;", "Type_Localised":"Guardian", "Count":1 }, { "Type":"$SAA_SignalType_Human;", "Type_Localised":"Menschlich", "Count":9 } ] }
        "FSSBodySignals" | "SAASignalsFound" => {
            //{ "timestamp":"2022-09-07T17:50:41Z", "event":"FSSBodySignals", "BodyName":"Synuefe EN-H d11-106 6 a", "BodyID":31, "SystemAddress":3652777380195, "Signals":[ { "Type":"$SAA_SignalType_Biological;", "Type_Localised":"Biologisch", "Count":1 }, { "Type":"$SAA_SignalType_Geological;", "Type_Localised":"Geologisch", "Count":3 } ] }
        }
        "FSSSignalDiscovered" => {}
        "SAAScanComplete" => {}
        "Scan" => {
            //TODO Learn tokio

            thread::spawn(||{
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async move {
                        let contract = get_contract().await;
                        if !json["BodyName"].to_string().contains("Belt Cluster") && !json["BodyName"].to_string().contains("Ring"){
                            if !json.has_key("StarType") {
                                //Planet (Body)
                                //Body
                                //{ "timestamp":"2022-10-16T23:51:17Z", "event":"Scan", "ScanType":"Detailed", "BodyName":"Ogmar A 6", "BodyID":40,
                                // "Parents":[ {"Star":1}, {"Null":0} ],
                                // "StarSystem":"Ogmar", "SystemAddress":84180519395914, "DistanceFromArrivalLS":3376.246435,
                                // "TidalLock":false, "TerraformState":"", "PlanetClass":"Sudarsky class I gas giant",
                                // "Atmosphere":"", "AtmosphereComposition":[ { "Name":"Hydrogen", "Percent":73.044167 }, { "Name":"Helium", "Percent":26.955832 } ],
                                // "Volcanism":"", "MassEM":24.477320, "Radius":22773508.000000, "SurfaceGravity":18.811067, "SurfaceTemperature":62.810730,
                                // "SurfacePressure":0.000000, "Landable":false, "SemiMajorAxis":1304152250289.916992, "Eccentricity":0.252734,
                                // "OrbitalInclination":156.334694, "Periapsis":269.403039, "OrbitalPeriod":990257555.246353, "AscendingNode":-1.479320,
                                // "MeanAnomaly":339.074691, "RotationPeriod":37417.276422, "AxialTilt":0.018931, "WasDiscovered":true, "WasMapped":true }
                                let function_call: FunctionCall<
                                    Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
                                    SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                                    (),
                                > = contract.register_planet(
                                    json["SystemAddress"].as_u64().unwrap(),
                                    json["BodyID"].as_u8().unwrap(),
                                    json["BodyName"].to_string(),
                                    json["WasDiscovered"].as_bool().unwrap(),
                                    json["WasMapped"].as_bool().unwrap(),
                                    extract_planet_properties(&json),
                                    extract_body_properties(&json)
                                );
                                execute_send_repeatable(function_call).await;
                            }else {
                                //Star
                                //{"AbsoluteMagnitude":8.518448,"Age_MY":446,"AxialTilt":0,"BodyID":0,"BodyName":"Hyades Sector BB-N b7-5",
                                // "DistanceFromArrivalLS":0,"Luminosity":"Va","Radius":374854272.0,"RotationPeriod":192595.293946,"ScanType":"AutoScan",
                                // "StarPos":[12.1875,-74.90625,-120.5],"StarSystem":"Hyades Sector BB-N b7-5","StarType":"M","StellarMass":0.394531,"Subclass":1,
                                // "SurfaceTemperature":3367.0,"SystemAddress":11666070513017,"WasDiscovered":true,"WasMapped":false,"event":"Scan","horizons":true,
                                // "odyssey":true,"timestamp":"2024-03-26T21:27:53Z"}
                                let function_call: FunctionCall<
                                    Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
                                    SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                                    (),
                                > = contract.register_star(
                                    json["SystemAddress"].as_u64().unwrap(),
                                    json["BodyID"].as_u8().unwrap(),
                                    json["BodyName"].to_string(),
                                    json["WasDiscovered"].as_bool().unwrap(),
                                    json["WasMapped"].as_bool().unwrap(),
                                    extract_star_properties(&json),
                                    extract_body_properties(&json)
                                );
                                execute_send_repeatable(function_call).await;
                            }
                        }else {
                            //TODO Interpret Belt Cluster and Ring
                        }
                    });
            });
        }
        //Planet scan with fss
        "ScanBaryCentre" => {}

        //Maintenance
        "RefuelAll" => {}
        "Resupply" => {}
        "Repair" => {}
        "BuyDrones" => {}
        "SellDrones" => {}
        "BuyAmmo" => {}
        //{ "timestamp":"2022-10-16T23:55:55Z", "event":"ReservoirReplenished", "FuelMain":30.905506, "FuelReservoir":1.070000 }
        "ReservoirReplenished" => {} //If reservoir needs to drain more fuel from main tank
        "RepairAll" => {}
        "RebootRepair" => {}
        "RestockVehicle" => {}

        //Docking
        "DockingRequested" => {}
        "DockingGranted" => {}
        "Docked" => {
            //{ "timestamp":"2023-09-09T23:59:09Z", "event":"CarrierJump", "Docked":true, "StationName":"Q2K-BHB", "StationType":"FleetCarrier", "MarketID":3704402432, "StationFaction":{ "Name":"FleetCarrier" }, "StationGovernment":"$government_Carrier;", "StationGovernment_Localised":"Privateigentum", "StationServices":[ "dock", "autodock", "commodities", "contacts", "exploration", "outfitting", "crewlounge", "rearm", "refuel", "repair", "shipyard", "engineer", "flightcontroller", "stationoperations", "stationMenu", "carriermanagement", "carrierfuel", "livery", "voucherredemption", "socialspace", "bartender", "vistagenomics" ], "StationEconomy":"$economy_Carrier;", "StationEconomy_Localised":"Privatunternehmen", "StationEconomies":[ { "Name":"$economy_Carrier;", "Name_Localised":"Privatunternehmen", "Proportion":1.000000 } ], "Taxi":false, "Multicrew":false, "StarSystem":"Plio Broae ML-D c2", "SystemAddress":637165713922, "StarPos":[2112.75000,719.12500,50162.93750], "SystemAllegiance":"", "SystemEconomy":"$economy_None;", "SystemEconomy_Localised":"n/v", "SystemSecondEconomy":"$economy_None;", "SystemSecondEconomy_Localised":"n/v", "SystemGovernment":"$government_None;", "SystemGovernment_Localised":"n/v", "SystemSecurity":"$GAlAXY_MAP_INFO_state_anarchy;", "SystemSecurity_Localised":"Anarchie", "Population":0, "Body":"Plio Broae ML-D c2", "BodyID":0, "BodyType":"Star" }
        }
        "Undocked" => {
            //{ "timestamp":"2023-09-09T18:29:17Z", "event":"Undocked", "StationName":"Q2K-BHB", "StationType":"FleetCarrier", "MarketID":3704402432, "Taxi":false, "Multicrew":false }
        }

        //Engineer
        "EngineerProgress" => {}
        "EngineerCraft" => {
            //{ "timestamp":"2023-12-05T20:54:13Z", "event":"EngineerCraft", "Slot":"PowerDistributor",
            // "Module":"int_powerdistributor_size7_class5",
            // "Ingredients":[
            // { "Name":"hybridcapacitors", "Name_Localised":"Hybridkondensatoren", "Count":1 },
            // { "Name":"industrialfirmware", "Name_Localised":"Gecrackte Industrie-Firmware", "Count":1 },
            // { "Name":"chemicalmanipulators", "Name_Localised":"Chemische Manipulatoren", "Count":1 } ],
            // "Engineer":"The Dweller", "EngineerID":300180, "BlueprintID":128673738, "BlueprintName":"PowerDistributor_HighFrequency",
            // "Level":4, "Quality":0.267800, "ExperimentalEffect":"special_powerdistributor_fast",
            // "ExperimentalEffect_Localised":"Superleiter",
            // "Modifiers":[
            // { "Label":"WeaponsCapacity", "Value":56.217598, "OriginalValue":61.000000, "LessIsGood":0 }, { "Label":"WeaponsRecharge", "Value":8.209770, "OriginalValue":6.100000, "LessIsGood":0 }, { "Label":"EnginesCapacity", "Value":37.785599, "OriginalValue":41.000000, "LessIsGood":0 }, { "Label":"EnginesRecharge", "Value":5.383456, "OriginalValue":4.000000, "LessIsGood":0 }, { "Label":"SystemsCapacity", "Value":37.785599, "OriginalValue":41.000000, "LessIsGood":0 }, { "Label":"SystemsRecharge", "Value":5.383456, "OriginalValue":4.000000, "LessIsGood":0 } ] }
        }
        "EngineerContribution" => {}

        //Ship management
        "Shipyard" => {}
        "StoredShips" => {}
        "ShipyardSwap" => {}
        "ShipLocker" => {}
        "ModuleBuy" => {}
        "Outfitting" => {}
        "ModuleInfo" => {}
        "StoredModules" => {}
        "DockingCancelled" => {}
        "ShipyardBuy" => {}
        "ShipyardNew" => {}
        "ShipyardTransfer" => {}
        "ModuleStore" => {}
        "ModuleSell" => {}
        "ModuleSellRemote" => {}
        "ModuleSwap" => {}

        //On foot
        "Backpack" => {}
        "BackpackChange" => {}
        "CollectItems" => {}
        "UpgradeSuit" => {}
        "Loadout" => {}
        "LoadoutEquipModule" => {}
        "SuitLoadout" => {}
        "UseConsumable" => {}
        "ScanOrganic" => {}

        //Market
        "MarketBuy" => {}
        "Market" => {}
        "MarketSell" => {}

        //SRV
        "LaunchSRV" => {}
        "DockSRV" => {}

        //Ship fight
        "ShipTargeted" => {}
        "UnderAttack" => {}
        "ShieldState" => {}
        "HullDamage" => {}

        //Cargo, Materials & Mining & Drones
        //{ "timestamp":"2022-09-07T20:08:23Z", "event":"Materials",
        // "Raw":[ { "Name":"sulphur", "Name_Localised":"Schwefel", "Count":300 }, { "Name":"manganese", "Name_Localised":"Mangan", "Count":236 }, { "Name":"vanadium", "Count":95 }, { "Name":"nickel", "Count":300 }, { "Name":"phosphorus", "Name_Localised":"Phosphor", "Count":296 }, { "Name":"iron", "Name_Localised":"Eisen", "Count":300 }, { "Name":"germanium", "Count":239 }, { "Name":"chromium", "Name_Localised":"Chrom", "Count":213 }, { "Name":"carbon", "Name_Localised":"Kohlenstoff", "Count":257 }, { "Name":"molybdenum", "Name_Localised":"Molibdän", "Count":153 }, { "Name":"cadmium", "Name_Localised":"Kadmium", "Count":13 }, { "Name":"selenium", "Name_Localised":"Selen", "Count":14 }, { "Name":"mercury", "Name_Localised":"Quecksilber", "Count":19 }, { "Name":"yttrium", "Count":22 }, { "Name":"zinc", "Name_Localised":"Zink", "Count":250 }, { "Name":"ruthenium", "Count":24 }, { "Name":"arsenic", "Name_Localised":"Arsen", "Count":24 }, { "Name":"tungsten", "Name_Localised":"Wolfram", "Count":75 }, { "Name":"tellurium", "Name_Localised":"Tellur", "Count":12 }, { "Name":"tin", "Name_Localised":"Zinn", "Count":131 }, { "Name":"antimony", "Name_Localised":"Antimon", "Count":45 }, { "Name":"niobium", "Name_Localised":"Niob", "Count":44 }, { "Name":"zirconium", "Count":48 }, { "Name":"technetium", "Count":39 }, { "Name":"lead", "Name_Localised":"Blei", "Count":90 }, { "Name":"boron", "Name_Localised":"Bor", "Count":14 }, { "Name":"polonium", "Count":8 } ],
        // "Manufactured":[ { "Name":"hybridcapacitors", "Name_Localised":"Hybridkondensatoren", "Count":197 }, { "Name":"heatdispersionplate", "Name_Localised":"Wärmeverteilungsplatte", "Count":67 }, { "Name":"gridresistors", "Name_Localised":"Gitterwiderstände", "Count":242 }, { "Name":"mechanicalequipment", "Name_Localised":"Mechanisches Equipment", "Count":220 }, { "Name":"fedcorecomposites", "Name_Localised":"Core Dynamics Kompositwerkstoffe", "Count":100 }, { "Name":"protoheatradiators", "Name_Localised":"Proto-Wärmestrahler", "Count":6 }, { "Name":"salvagedalloys", "Name_Localised":"Geborgene Legierungen", "Count":300 }, { "Name":"highdensitycomposites", "Name_Localised":"Komposite hoher Dichte", "Count":200 }, { "Name":"mechanicalscrap", "Name_Localised":"Mechanischer Schrott", "Count":64 }, { "Name":"chemicalprocessors", "Name_Localised":"Chemische Prozessoren", "Count":250 }, { "Name":"focuscrystals", "Name_Localised":"Laserkristalle", "Count":200 }, { "Name":"imperialshielding", "Name_Localised":"Imperiale Schilde", "Count":53 }, { "Name":"precipitatedalloys", "Name_Localised":"Gehärtete Legierungen", "Count":200 }, { "Name":"galvanisingalloys", "Name_Localised":"Galvanisierende Legierungen", "Count":250 }, { "Name":"shieldingsensors", "Name_Localised":"Schildsensoren", "Count":200 }, { "Name":"chemicaldistillery", "Name_Localised":"Chemiedestillerie", "Count":200 }, { "Name":"heatconductionwiring", "Name_Localised":"Wärmeleitungsverdrahtung", "Count":128 }, { "Name":"phasealloys", "Name_Localised":"Phasenlegierungen", "Count":195 }, { "Name":"wornshieldemitters", "Name_Localised":"Gebrauchte Schildemitter", "Count":300 }, { "Name":"shieldemitters", "Name_Localised":"Schildemitter", "Count":250 }, { "Name":"mechanicalcomponents", "Name_Localised":"Mechanische Komponenten", "Count":11 }, { "Name":"compoundshielding", "Name_Localised":"Verbundschilde", "Count":150 }, { "Name":"protolightalloys", "Name_Localised":"Leichte Legierungen (Proto)", "Count":145 }, { "Name":"refinedfocuscrystals", "Name_Localised":"Raffinierte Laserkristalle", "Count":150 }, { "Name":"heatexchangers", "Name_Localised":"Wärmeaustauscher", "Count":6 }, { "Name":"conductiveceramics", "Name_Localised":"Elektrokeramiken", "Count":44 }, { "Name":"uncutfocuscrystals", "Name_Localised":"Fehlerhafte Fokuskristalle", "Count":250 }, { "Name":"temperedalloys", "Name_Localised":"Vergütete Legierungen", "Count":92 }, { "Name":"basicconductors", "Name_Localised":"Einfache Leiter", "Count":140 }, { "Name":"crystalshards", "Name_Localised":"Kristallscherben", "Count":288 }, { "Name":"unknownenergycell", "Name_Localised":"Thargoiden-Energiezelle", "Count":171 }, { "Name":"unknowntechnologycomponents", "Name_Localised":"Technologiekomponenten der Thargoiden", "Count":150 }, { "Name":"unknownenergysource", "Name_Localised":"Sensorenfragment", "Count":100 }, { "Name":"unknowncarapace", "Name_Localised":"Thargoiden-Krustenschale", "Count":220 }, { "Name":"unknownorganiccircuitry", "Name_Localised":"Organischer Schaltkreis der Thargoiden", "Count":100 }, { "Name":"chemicalmanipulators", "Name_Localised":"Chemische Manipulatoren", "Count":72 }, { "Name":"exquisitefocuscrystals", "Name_Localised":"Erlesene Laserkristalle", "Count":89 }, { "Name":"configurablecomponents", "Name_Localised":"Konfigurierbare Komponenten", "Count":36 }, { "Name":"heatvanes", "Name_Localised":"Wärmeleitbleche", "Count":1 }, { "Name":"biotechconductors", "Name_Localised":"Biotech-Leiter", "Count":57 }, { "Name":"conductivepolymers", "Name_Localised":"Leitfähige Polymere", "Count":5 }, { "Name":"thermicalloys", "Name_Localised":"Thermische Legierungen", "Count":150 }, { "Name":"conductivecomponents", "Name_Localised":"Leitfähige Komponenten", "Count":169 }, { "Name":"fedproprietarycomposites", "Name_Localised":"Kompositwerkstoffe", "Count":150 }, { "Name":"electrochemicalarrays", "Name_Localised":"Elektrochemische Detektoren", "Count":133 }, { "Name":"compactcomposites", "Name_Localised":"Kompaktkomposite", "Count":101 }, { "Name":"filamentcomposites", "Name_Localised":"Filament-Komposite", "Count":250 }, { "Name":"chemicalstorageunits", "Name_Localised":"Lagerungseinheiten für Chemiestoffe", "Count":57 }, { "Name":"protoradiolicalloys", "Name_Localised":"Radiologische Legierungen (Proto)", "Count":39 }, { "Name":"guardian_powercell", "Name_Localised":"Guardian-Energiezelle", "Count":300 }, { "Name":"guardian_powerconduit", "Name_Localised":"Guardian-Energieleiter", "Count":250 }, { "Name":"guardian_techcomponent", "Name_Localised":"Guardian-Technologiekomponenten", "Count":160 }, { "Name":"guardian_sentinel_weaponparts", "Name_Localised":"Guardian-Wache-Waffenteile", "Count":200 }, { "Name":"pharmaceuticalisolators", "Name_Localised":"Pharmazeutische Isolatoren", "Count":27 }, { "Name":"militarygradealloys", "Name_Localised":"Militärqualitätslegierungen", "Count":63 }, { "Name":"guardian_sentinel_wreckagecomponents", "Name_Localised":"Guardian-Wrackteilkomponenten", "Count":300 }, { "Name":"heatresistantceramics", "Name_Localised":"Hitzefeste Keramik", "Count":87 }, { "Name":"polymercapacitors", "Name_Localised":"Polymerkondensatoren", "Count":91 }, { "Name":"tg_biomechanicalconduits", "Name_Localised":"Biomechanische Leiter", "Count":105 }, { "Name":"tg_wreckagecomponents", "Name_Localised":"Wrackteilkomponenten", "Count":144 }, { "Name":"tg_weaponparts", "Name_Localised":"Waffenteile", "Count":135 }, { "Name":"tg_propulsionelement", "Name_Localised":"Schubantriebelemente", "Count":100 }, { "Name":"militarysupercapacitors", "Name_Localised":"Militärische Superkondensatoren", "Count":1 }, { "Name":"improvisedcomponents", "Name_Localised":"Behelfskomponenten", "Count":4 } ],
        // "Encoded":[ { "Name":"shielddensityreports", "Name_Localised":"Untypische Schildscans ", "Count":200 }, { "Name":"shieldcyclerecordings", "Name_Localised":"Gestörte Schildzyklus-Aufzeichnungen", "Count":234 }, { "Name":"encryptedfiles", "Name_Localised":"Ungewöhnliche verschlüsselte Files", "Count":92 }, { "Name":"bulkscandata", "Name_Localised":"Anormale Massen-Scan-Daten", "Count":192 }, { "Name":"decodedemissiondata", "Name_Localised":"Entschlüsselte Emissionsdaten", "Count":112 }, { "Name":"encryptioncodes", "Name_Localised":"Getaggte Verschlüsselungscodes", "Count":33 }, { "Name":"shieldsoakanalysis", "Name_Localised":"Inkonsistente Schildleistungsanalysen", "Count":250 }, { "Name":"scanarchives", "Name_Localised":"Unidentifizierte Scan-Archive", "Count":112 }, { "Name":"disruptedwakeechoes", "Name_Localised":"Atypische FSA-Stör-Aufzeichnungen", "Count":228 }, { "Name":"archivedemissiondata", "Name_Localised":"Irreguläre Emissionsdaten", "Count":65 }, { "Name":"legacyfirmware", "Name_Localised":"Spezial-Legacy-Firmware", "Count":78 }, { "Name":"scrambledemissiondata", "Name_Localised":"Außergewöhnliche verschlüsselte Emissionsdaten", "Count":84 }, { "Name":"encodedscandata", "Name_Localised":"Divergente Scandaten", "Count":30 }, { "Name":"fsdtelemetry", "Name_Localised":"Anormale FSA-Telemetrie", "Count":123 }, { "Name":"wakesolutions", "Name_Localised":"Seltsame FSA-Zielorte", "Count":93 }, { "Name":"emissiondata", "Name_Localised":"Unerwartete Emissionsdaten", "Count":142 }, { "Name":"shieldpatternanalysis", "Name_Localised":"Abweichende Schildeinsatz-Analysen", "Count":78 }, { "Name":"scandatabanks", "Name_Localised":"Scan-Datenbanken unter Verschluss", "Count":68 }, { "Name":"consumerfirmware", "Name_Localised":"Modifizierte Consumer-Firmware", "Count":48 }, { "Name":"symmetrickeys", "Name_Localised":"Offene symmetrische Schlüssel", "Count":24 }, { "Name":"shieldfrequencydata", "Name_Localised":"Verdächtige Schildfrequenz-Daten", "Count":50 }, { "Name":"compactemissionsdata", "Name_Localised":"Anormale kompakte Emissionsdaten", "Count":18 }, { "Name":"adaptiveencryptors", "Name_Localised":"Adaptive Verschlüsselungserfassung", "Count":64 }, { "Name":"encryptionarchives", "Name_Localised":"Atypische Verschlüsselungsarchive", "Count":63 }, { "Name":"dataminedwake", "Name_Localised":"FSA-Daten-Cache-Ausnahmen", "Count":19 }, { "Name":"securityfirmware", "Name_Localised":"Sicherheits-Firmware-Patch", "Count":29 }, { "Name":"embeddedfirmware", "Name_Localised":"Modifizierte integrierte Firmware", "Count":58 }, { "Name":"tg_residuedata", "Name_Localised":"Thargoiden-Rückstandsdaten", "Count":55 }, { "Name":"tg_compositiondata", "Name_Localised":"Materialzusammensetzungsdaten der Thargoiden", "Count":49 }, { "Name":"tg_structuraldata", "Name_Localised":"Thargoiden-Strukturdaten", "Count":49 }, { "Name":"unknownshipsignature", "Name_Localised":"Thargoiden-Schiffssignatur", "Count":37 }, { "Name":"unknownwakedata", "Name_Localised":"Thargoiden-Sogwolkendaten", "Count":55 }, { "Name":"ancienthistoricaldata", "Name_Localised":"Gamma-Muster-Obeliskendaten", "Count":150 }, { "Name":"ancienttechnologicaldata", "Name_Localised":"Epsilon-Muster-Obeliskendaten", "Count":150 }, { "Name":"ancientbiologicaldata", "Name_Localised":"Alpha-Muster-Obeliskendaten", "Count":150 }, { "Name":"ancientlanguagedata", "Name_Localised":"Delta-Muster-Obeliskendaten", "Count":150 }, { "Name":"ancientculturaldata", "Name_Localised":"Beta-Muster-Obeliskendaten", "Count":150 }, { "Name":"classifiedscandata", "Name_Localised":"Geheimes Scan-Fragment", "Count":18 }, { "Name":"hyperspacetrajectories", "Name_Localised":"Exzentrische Hyperraum-Routen", "Count":104 }, { "Name":"guardian_weaponblueprint", "Name_Localised":"Guardian-Waffenbauplanfragment", "Count":4 }, { "Name":"guardian_moduleblueprint", "Name_Localised":"Guardian-Modulbauplanfragment", "Count":7 }, { "Name":"guardian_vesselblueprint", "Name_Localised":"Guardian-Schiffsbauplanfragment", "Count":8 }, { "Name":"tg_shipflightdata", "Name_Localised":"Schiffsflugdaten", "Count":18 }, { "Name":"tg_shipsystemsdata", "Name_Localised":"Schiffssysteme-Daten", "Count":45 } ] }
        "Materials" => {}
        "Cargo" => {}
        "MaterialCollected" => {
            //{ "timestamp":"2023-12-05T19:44:43Z", "event":"MaterialCollected", "Category":"Manufactured", "Name":"shieldemitters", "Name_Localised":"Schildemitter", "Count":3 }            let material_category = json["Category"].to_string();
        }
        "Synthesis" => {}
        "EjectCargo" => {}
        "DropItems" => {}
        "LaunchDrone" => {}
        "MiningRefined" => {}
        "ProspectedAsteroid" => {
            //{ "timestamp":"2023-06-05T12:05:12Z", "event":"ProspectedAsteroid", "Materials":[ { "Name":"rutile", "Name_Localised":"Rutil", "Proportion":35.986309 }, { "Name":"Bauxite", "Name_Localised":"Bauxit", "Proportion":13.713245 } ], "Content":"$AsteroidMaterialContent_Low;", "Content_Localised":"Materialgehalt: Niedrig", "Remaining":100.000000 }
        }
        "CargoTransfer" => {}
        "CollectCargo" => {}

        //Mission and Redeeming
        "Missions" => {}
        "MissionAccepted" => {}
        "MissionRedirected" => {}
        "MissionCompleted" => {}
        "RedeemVoucher" => {}
        "Bounty" => {}
        "NpcCrewPaidWage" => {}
        "PayFines" => {}
        "MissionAbandoned" => {}
        "MissionFailed" => {}
        "PayBounties" => {}
        "SellOrganicData" => {}

        //Carrier
        "CarrierStats" => {}
        "CarrierJumpRequest" => {}
        "CarrierTradeOrder" => {}
        "CarrierFinance" => {}
        "CarrierJumpCancelled" => {}
        "CarrierDepositFuel" => {}
        "CarrierDockingPermission" => {}
        "CarrierCrewServices" => {}
        "CarrierModulePack" => {}
        "CarrierBankTransfer" => {}

        //Dropship
        "BookDropship" => {}
        "DropshipDeploy" => {}

        //Wing
        "WingInvite" => {}
        "WingJoin" => {}
        "WingAdd" => {}
        "WingLeave" => {}

        //Crew
        "CrewMemberQuits" => {}
        "CrewMemberRoleChange" => {}
        "CrewMemberJoins" => {}
        "EndCrewSession" => {}

        "SellMicroResources" => {}
        "TradeMicroResources" => {}
        "FuelScoop" => {}
        "ReceiveText" => {}
        "Friends" => {}
        "Scanned" => {}
        "LoadGame" => {}
        "SquadronStartup" => {}
        "Music" => {}
        "CodexEntry" => {}
        "Rank" => {}
        "Progress" => {}
        "Reputation" => {}
        "Statistics" => {}
        "Commander" => {}
        "PowerplaySalary" => {}
        "Powerplay" => {}
        "CommitCrime" => {}
        "DockingDenied" => {}
        "HeatWarning" => {}
        "FactionKillBond" => {}
        "MultiSellExplorationData" => {}
        "SwitchSuitLoadout" => {}
        "MaterialTrade" => {
            //{ "timestamp":"2023-12-05T19:23:23Z", "event":"MaterialTrade", "MarketID":3223208960, "TraderType":"manufactured",
            // "Paid":{ "Material":"fedcorecomposites", "Material_Localised":"Core Dynamics Kompositwerkstoffe", "Category":"Manufactured", "Quantity":6 },
            // "Received":{ "Material":"protoradiolicalloys", "Material_Localised":"Radiologische Legierungen (Proto)", "Category":"Manufactured", "Quantity":1 } }
        }
        "CommunityGoal" => {}
        "ModuleRetrieve" => {}
        "FetchRemoteModule" => {}
        "SendText" => {}
        "SearchAndRescue" => {}
        "HeatDamage" => {}
        "CommunityGoalReward" => {}
        "NavBeaconScan" => {}
        "USSDrop" => {}
        "Interdicted" => {}
        "Promotion" => {}
        "RepairDrone" => {}
        "DataScanned" => {}
        "DatalinkScan" => {}
        "DatalinkVoucher" => {}
        "CockpitBreached" => {}
        "SystemsShutdown" => {}
        "Screenshot" => {}
        "UpgradeWeapon" => {}
        "PowerplayFastTrack" => {}
        "PowerplayCollect" => {}
        "PowerplayDeliver" => {}
        "BookTaxi" => {}
        "SharedBookmarkToSquadron" => {}
        "MaterialDiscovered" => {}
        "SetUserShipName" => {}
        "FCMaterials" => {}
        "CommunityGoalJoin" => {}
        "SupercruiseDestinationDrop" => {}
        "JetConeBoost" => {}
        "AsteroidCracked" => {}
        "EscapeInterdiction" => {}
        "TechnologyBroker" => {}
        "NavBeaconDetail" => {}

        //Jesus
        "Died" => {}
        "Resurrect" => {}
        "SelfDestruct" => {}

        "Fileheader" => {}
        "Shutdown" => {}
        "null" => {}
        "" => {}
        _ => {
            println!("UNKNOWN EVENT:{}", json["event"]);
        }
    }
}

fn extract_planet_properties(json: &JsonValue) -> PlanetProperties {
    PlanetProperties {
        atmosphere: json["Atmosphere"].to_string(),
        class: json["PlanetClass"].to_string(),
        landable: json["Landable"].as_bool().unwrap_or(false),
        terraform_state: json["TerraformState"].to_string(),
        volcanism: json["Volcanism"].to_string(),
        tidal_lock: json["TidalLock"].as_bool().unwrap_or_else(|| panic!("Tidal Lock not parseable {}", json)),
        mass_em: edcas_contract::Floating{
            decimal: json["MassEM"].to_string().replace('.',"").parse().unwrap(),
            floating_point: json["MassEM"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
        surface_gravity: edcas_contract::Floating{
            decimal: json["SurfaceGravity"].to_string().replace('.',"").parse().unwrap(),
            floating_point: json["SurfaceGravity"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
        surface_pressure: edcas_contract::Floating{
            decimal: json["SurfacePressure"].to_string().replace('.',"").parse().unwrap(),
            floating_point: json["SurfacePressure"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
        ascending_node: edcas_contract::Floating {
            decimal: json["AscendingNode"].to_string().replace('.', "").parse().unwrap_or(0),
            floating_point: json["AscendingNode"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
        eccentricity: edcas_contract::Floating {
            decimal: json["Eccentricity"].to_string().replace('.', "").parse().unwrap_or_else(|_| panic!("Eccentricity invalid parse: {}", json)),
            floating_point: json["Eccentricity"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
        mean_anomaly: edcas_contract::Floating {
            decimal: json["MeanAnomaly"].to_string().replace('.', "").parse().unwrap_or(0),
            floating_point: json["MeanAnomaly"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
        orbital_inclination: edcas_contract::Floating {
            decimal: json["OrbitalInclination"].to_string().replace('.', "").parse().unwrap(),
            floating_point: json["OrbitalInclination"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
        orbital_period: edcas_contract::Floating {
            decimal: json["OrbitalPeriod"].to_string().replace('.', "").parse().unwrap(),
            floating_point: json["OrbitalPeriod"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
        periapsis: edcas_contract::Floating {
            decimal: json["Periapsis"].to_string().replace('.', "").parse().unwrap(),
            floating_point: json["Periapsis"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
        semi_major_axis: edcas_contract::Floating {
            decimal: json["SemiMajorAxis"].to_string().replace('.', "").parse().unwrap(),
            floating_point: json["SemiMajorAxis"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
    }
}
fn extract_star_properties(json: &JsonValue) -> StarProperties {
    StarProperties {
        subclass: json["Subclass"].as_u8().unwrap(),
        age_my: json["Age_MY"].as_u16().unwrap(),
        type_: json["StarType"].to_string(),
        luminosity: json["Luminosity"].to_string(),
        stellar_mass: edcas_contract::Floating{
            decimal: json["StellarMass"].to_string().replace('.',"").parse().unwrap(),
            floating_point: json["StellarMass"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
        absolute_magnitude: edcas_contract::Floating{
            decimal: json["AbsoluteMagnitude"].to_string().replace('.',"").parse().unwrap_or_else(|_| panic!("AbsoluteMagnitude parse error: {}",json)),
            floating_point: json["AbsoluteMagnitude"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
    }
}
fn extract_body_properties(json: &JsonValue) -> BodyProperties {
    BodyProperties {
        radius: edcas_contract::Floating {
            decimal: json["Radius"].to_string().replace('.', "").parse().unwrap(),
            floating_point: json["Radius"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
        distance_from_arrival_ls: edcas_contract::Floating {
            decimal: json["DistanceFromArrivalLS"].to_string().replace('.', "").parse().unwrap(),
            floating_point: json["DistanceFromArrivalLS"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
        axial_tilt: edcas_contract::Floating {
            decimal: json["AxialTilt"].to_string().replace('.', "").parse().unwrap(),
            floating_point: json["AxialTilt"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
        rotation_period: edcas_contract::Floating {
            decimal: json["RotationPeriod"].to_string().replace('.', "").parse().unwrap(),
            floating_point: json["RotationPeriod"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
        surface_temperature: edcas_contract::Floating {
            decimal: json["SurfaceTemperature"].to_string().replace('.', "").parse().unwrap(),
            floating_point: json["SurfaceTemperature"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
        },
    }
}
#[derive(PartialEq)]
enum SendError{
    RepeatableError(String), NonRepeatableError(String)
}
async fn execute_send_repeatable(function_call: FunctionCall<Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>, SignerMiddleware<Provider<Http>, Wallet<SigningKey>>, ()>) {
    while let Err(err) = execute_send(function_call.clone()).await {
        match err {
            RepeatableError(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            NonRepeatableError(_) => {break}
        }
    }
}
async fn execute_send(function_call: FunctionCall<Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>, SignerMiddleware<Provider<Http>, Wallet<SigningKey>>, ()>) -> Result<H256,SendError>{
    match function_call.legacy().send().await {
        Ok(pending) => {
            match pending.await {
                Ok(receipt) => {
                    if let Some(receipt) = receipt {
                        if let Some(hash) = receipt.block_hash {
                            println!("{:?}",hash);
                            Ok(hash)
                        }else {
                            Err(NonRepeatableError("Receipt without hash".into()))
                        }
                    }else {
                        Err(NonRepeatableError("No Receipt".into()))
                    }
                }
                Err(err) => {
                    match err {
                        ProviderError::JsonRpcClientError(err) => {
                            eprintln!("JsonRpcClientError: {}",err.to_string());
                            Err(RepeatableError(err.to_string()))
                        }
                        ProviderError::EnsError(err) => {
                            eprintln!("EnsError: {}",err.to_string());
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::EnsNotOwned(err) => {
                            eprintln!("EnsNotOwned: {}",err.to_string());
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::SerdeJson(err) => {
                            eprintln!("SerdeJson: {}",err.to_string());
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::HexError(err) => {
                            eprintln!("HexError: {}",err.to_string());
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::HTTPError(err) => {
                            eprintln!("HTTPError: {}",err.to_string());
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::CustomError(err) => {
                            eprintln!("CustomError: {}",err.to_string());
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::UnsupportedRPC => {
                            eprintln!("UnsupportedRPC: {}", "UnsupportedRPC".to_string());
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::UnsupportedNodeClient => {
                            eprintln!("UnsupportedNodeClient: {}", "UnsupportedNodeClient".to_string());
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::SignerUnavailable => {
                            eprintln!("SignerUnavailable: {}", "SignerUnavailable".to_string());
                            Err(NonRepeatableError(err.to_string()))
                        }
                    }

                }
            }
        }
        Err(err) => {
            match err {
                ContractError::Revert(err) => {
                    let message = get_revert_message(err);
                    eprintln!("Revert: {}", message);
                    Err(NonRepeatableError(message))
                }
                ContractError::DecodingError(err) => {
                    eprintln!("DecodingError: {}",err.to_string());
                    Err(NonRepeatableError(err.to_string()))
                }
                ContractError::AbiError(err) => {
                    eprintln!("AbiError: {}",err.to_string());
                    Err(NonRepeatableError(err.to_string()))
                }
                ContractError::DetokenizationError(err) => {
                    eprintln!("DetokenizationError: {}",err.to_string());
                    Err(NonRepeatableError(err.to_string()))
                }
                ContractError::MiddlewareError { e } => {
                    eprintln!("MiddlewareError: {}",e.to_string());
                    Err(RepeatableError(e.to_string()))
                }
                ContractError::ProviderError { e } => {
                    eprintln!("ProviderError: {}",e);
                    Err(NonRepeatableError(e.to_string()))
                }
                ContractError::ConstructorError => {
                    eprintln!("{}", "ConstructorError".to_string());
                    Err(NonRepeatableError(err.to_string()))
                }
                ContractError::ContractNotDeployed => {
                    eprintln!("{}", "ContractNotDeployed".to_string());
                    Err(NonRepeatableError(err.to_string()))
                }
            }

        }
    }
}

async fn get_contract() -> edcas_contract::EDCAS<SignerMiddleware<Provider<Http>, LocalWallet>> {
    let provider = Provider::connect(env::var("EVM_URL").unwrap().as_str()).await;

    let wallet: LocalWallet = env::var("PRIVATE_KEY").unwrap()
        .parse::<LocalWallet>()
        .unwrap();

    let mut result = SignerMiddleware::new_with_provider_chain(provider.clone(), wallet.clone())
        .await;
    let mut retries = 0;
    while result.is_err() && retries < env::var("RETRY_TIMEOUT").unwrap_or("100".into()).parse().unwrap_or(100) {
        retries += 1;
        tokio::time::sleep(Duration::from_secs(env::var("DURATION_TIMEOUT").unwrap_or("5".into()).parse().unwrap_or(5))).await;
        result = SignerMiddleware::new_with_provider_chain(provider.clone(), wallet.clone())
            .await;
    }

    let client = result.unwrap();
    
    let edcas_address = env::var("SC_ADDRESS").unwrap()
        .parse::<Address>()
        .unwrap();

    edcas_contract::EDCAS::new(edcas_address, Arc::new(client.clone()))
}

fn get_revert_message(bytes: Bytes) -> String {
    match hex::encode(&bytes).as_str() { 
        "08c379a00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000001953797374656d206e6f7420726567697374657265642079657400000000000000" => {
            String::from("System not registered yet")
        }
        &_ => {
            hex::encode(&bytes).replace("000000",".")
        } 
    }
}