use std::sync::mpsc::channel;
use std::sync::Arc;
use std::{env, thread};
use std::time::Duration;
use chrono::DateTime;

use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use json::JsonValue;
use crate::edcas_contract::{BodyProperties, Faction, PlanetProperties, StarProperties};
use crate::eddn_adapter::EddnAdapter;
use crate::SendError::{NonRepeatableError, RepeatableError};

mod eddn_adapter;
mod edcas_contract;

//abigen!(EDCAS_Contract, "./contracts/EDCAS.abi");
type ContractCall = FunctionCall<
    Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
    ()>;
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
        "FSDJump" => {
            process_jump(json.clone());
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
                                    extract_body_properties(&json),
                                    DateTime::parse_from_rfc3339(
                                        json["timestamp"].as_str().unwrap(),
                                    )
                                        .unwrap()
                                        .timestamp()
                                        .into()
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
                                    extract_body_properties(&json),
                                    DateTime::parse_from_rfc3339(
                                        json["timestamp"].as_str().unwrap(),
                                    )
                                        .unwrap()
                                        .timestamp()
                                        .into()
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
            //{ "timestamp":"2024-04-02T21:22:42Z", "event":"Docked", "StationName":"Q2K-BHB", "StationType":"FleetCarrier", "Taxi":false, "Multicrew":false, 
            // "StarSystem":"Dulos", "SystemAddress":13865362204089, "MarketID":3704402432, 
            // "StationFaction":{ "Name":"FleetCarrier" }, "StationGovernment":"$government_Carrier;", "StationGovernment_Localised":"Private Ownership", 
            // "StationServices":[ "dock", "autodock", "commodities", "contacts", "exploration", "outfitting", "crewlounge", "rearm", "refuel", "repair", "shipyard", "engineer", "flightcontroller", "stationoperations", "stationMenu", "carriermanagement", "carrierfuel", "livery", "voucherredemption", "socialspace", "bartender", "vistagenomics" ], 
            // "StationEconomy":"$economy_Carrier;", "StationEconomy_Localised":"Private Enterprise", 
            // "StationEconomies":[ { "Name":"$economy_Carrier;", "Name_Localised":"Private Enterprise", "Proportion":1.000000 } ], 
            // "DistFromStarLS":0.000000, "LandingPads":{ "Small":4, "Medium":4, "Large":8 } }

            //{ "timestamp":"2024-04-02T19:42:24Z", "event":"Docked", "StationName":"Milnor Station", "StationType":"Ocellus", "Taxi":false, "Multicrew":false, 
            // "StarSystem":"Dulos", "SystemAddress":13865362204089, "MarketID":3223819264, 
            // "StationFaction":{ "Name":"The Sovereign Justice Collective", "FactionState":"Bust" }, 
            // "StationGovernment":"$government_Dictatorship;", "StationGovernment_Localised":"Dictatorship", 
            // "StationServices":[ "dock", "autodock", "commodities", "contacts", "exploration", "missions", "outfitting", "crewlounge", "rearm", "refuel", "repair", "shipyard", "tuning", "engineer", "missionsgenerated", "flightcontroller", "stationoperations", "powerplay", "searchrescue", "stationMenu", "shop", "livery", "socialspace", "bartender", "vistagenomics", "pioneersupplies", "apexinterstellar", "frontlinesolutions" ], 
            // "StationEconomy":"$economy_Refinery;", "StationEconomy_Localised":"Refinery", 
            // "StationEconomies":[ { "Name":"$economy_Refinery;", "Name_Localised":"Refinery", "Proportion":1.000000 } ], 
            // "DistFromStarLS":20.275191, "LandingPads":{ "Small":11, "Medium":13, "Large":6 } }
            thread::spawn(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async move {
                        let mut services = String::new();
                        for entry in 0..json["StationServices"].len() {
                            if !services.is_empty(){
                                services.push_str(",");
                            }
                            services.push_str(json["StationServices"][entry].as_str().unwrap());
                        }
                        let contract = get_contract().await;
                        if json["StationType"].as_str().unwrap() == "FleetCarrier"{
                            
                            let function_call: ContractCall
                            = contract.register_carrier(
                                json["MarketID"].as_u64().unwrap(),
                                "Fleet Carrier".to_string(),
                                json["StationName"].as_str().unwrap().to_string(),
                                services,
                                "".to_string(),
                                false,
                                DateTime::parse_from_rfc3339(
                                    json["timestamp"].as_str().unwrap(),
                                )
                                    .unwrap()
                                    .timestamp()
                                    .into()
                            );
                            //execute_send(function_call).await;
                            execute_send_repeatable(function_call).await;

                            let function_call: ContractCall
                                = contract.report_carrier_location(
                                json["MarketID"].as_u64().unwrap(),
                                json["StarSystem"].as_str().unwrap().to_string(),
                                "Unkown".to_string(),
                                DateTime::parse_from_rfc3339(
                                    json["timestamp"].as_str().unwrap(),
                                )
                                    .unwrap()
                                    .timestamp()
                                    .into()
                            );
                            //execute_send(function_call).await;
                            execute_send_repeatable(function_call).await;
                        }else {
                            let function_call: ContractCall = contract.register_station(
                                json["MarketID"].as_u64().unwrap(),
                                json["StationName"].as_str().unwrap().to_string(),
                                json["StationType"].as_str().unwrap().to_string(),
                                json["SystemAddress"].as_u64().unwrap(),
                                json["StarSystem"].as_str().unwrap().to_string(),
                                Faction{
                                    name: json["StationFaction"]["Name"].as_str().unwrap().to_string(),
                                    state: json["StationFaction"]["FactionState"].as_str().unwrap_or("").to_string(),
                                },
                                json["StationGovernment"].as_str().unwrap().to_string(),
                                json["StationEconomy"].as_str().unwrap().to_string(),
                                services, 
                                edcas_contract::Floating{
                                    decimal: json["DistFromStarLS"].to_string().replace('.',"").parse().unwrap(),
                                    floating_point: json["DistFromStarLS"].to_string().split('.').nth(1).unwrap_or("").len() as u8,
                                },
                                json["LandingPads"].to_string(),
                                DateTime::parse_from_rfc3339(json["timestamp"].as_str().unwrap()).unwrap().timestamp().into()
                            );
                            //execute_send(function_call).await;
                            execute_send_repeatable(function_call).await;
                        }
                        
                    });
            });
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
        "CarrierJumpRequest" => {
            //{
            //     "timestamp": "2020-04-20T09:30:58Z",
            //     "event": "CarrierJumpRequest",
            //     "CarrierID": 3700005632,
            //     "SystemName": "Paesui Xena",
            //     "Body": "Paesui Xena A",
            //     "SystemAddress": 7269634680241,
            //     "BodyID": 1,
            //     "DepartureTime":"2020-04-20T09:45:00Z"
            // }
            thread::spawn(|| {
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
                        > = contract.emit_carrier_jump(
                            json["CarrierID"].as_u64().unwrap(),
                            json["SystemName"].as_str().unwrap().to_string(),
                            json["Body"].as_str().unwrap().to_string(),
                            DateTime::parse_from_rfc3339(json["DepartureTime"].as_str().unwrap()).unwrap().timestamp().into()
                        );
                        //execute_send(function_call).await;
                        execute_send_repeatable(function_call).await;
                    });
            });
        }
        "CarrierJumpCancelled" => {
            thread::spawn(|| {
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
                        > = contract.cancel_carrier_jump(
                            json["CarrierID"].as_u64().unwrap()
                        );
                        //execute_send(function_call).await;
                        execute_send_repeatable(function_call).await;
                    });
            });
        }
        "CarrierJump" => {
            //{ "timestamp":"2023-09-09T23:59:09Z", "event":"CarrierJump", "Docked":true, "StationName":"Q2K-BHB", "StationType":"FleetCarrier",
            // "MarketID":3704402432, "StationFaction":{ "Name":"FleetCarrier" }, "StationGovernment":"$government_Carrier;",
            // "StationGovernment_Localised":"Privateigentum", "StationServices":[ "dock", "autodock", "commodities", "contacts", "exploration", "outfitting", "crewlounge", "rearm", "refuel", "repair", "shipyard", "engineer", "flightcontroller", "stationoperations", "stationMenu", "carriermanagement", "carrierfuel", "livery", "voucherredemption", "socialspace", "bartender", "vistagenomics" ],
            // "StationEconomy":"$economy_Carrier;", "StationEconomy_Localised":"Privatunternehmen",
            // "StationEconomies":[ { "Name":"$economy_Carrier;", "Name_Localised":"Privatunternehmen", "Proportion":1.000000 } ], "Taxi":false, "Multicrew":false,
            // "StarSystem":"Plio Broae ML-D c2", "SystemAddress":637165713922, "StarPos":[2112.75000,719.12500,50162.93750], "SystemAllegiance":"",
            // "SystemEconomy":"$economy_None;", "SystemEconomy_Localised":"n/v", "SystemSecondEconomy":"$economy_None;", "SystemSecondEconomy_Localised":"n/v",
            // "SystemGovernment":"$government_None;", "SystemGovernment_Localised":"n/v", "SystemSecurity":"$GAlAXY_MAP_INFO_state_anarchy;",
            // "SystemSecurity_Localised":"Anarchie", "Population":0, "Body":"Plio Broae ML-D c2", "BodyID":0, "BodyType":"Star" }
            process_jump(json.clone());
        }
        "CarrierBuy" => {
            //{
            //     "timestamp": "2020-03-11T15:31:46Z",
            //     "event": "CarrierBuy",
            //     "CarrierID": 3700029440,
            //     "BoughtAtMarket": 3221301504,
            //     "Location": "Kakmbutan",
            //     "SystemAddress": 3549513615723,
            //     "Price": 4875000000,
            //     "Variant": "CarrierDockB",
            //     "Callsign": "P07-V3L"
            // }
            thread::spawn(|| {
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
                        > = contract.register_carrier(
                            json["CarrierID"].as_u64().unwrap(),
                            "Fleet Carrier".to_string(),
                            json["Callsign"].as_str().unwrap().to_string(),
                            "".to_string(),
                            "".to_string(),
                            false,
                            DateTime::parse_from_rfc3339(
                                json["timestamp"].as_str().unwrap(),
                            )
                                .unwrap()
                                .timestamp()
                                .into()
                        );
                        //execute_send(function_call).await;
                        execute_send_repeatable(function_call).await;
                    });
            });
        }
        "CarrierDecommission" => {
            //{
            //     "timestamp": "2020-03-11T15:12:26Z",
            //     "event": "CarrierDecommission",
            //     "CarrierID": 3700005632,
            //     "ScrapRefund": 1746872629,
            //     "ScrapTime": 1584601200
            // }
        }
        "CarrierStats" => {
            //{ "timestamp":"2024-03-31T18:14:39Z", "event":"CarrierStats", "CarrierID":3704402432, "Callsign":"Q2K-BHB", "Name":"FUXBAU", 
            // "DockingAccess":"all", "AllowNotorious":true, "FuelLevel":885, "JumpRangeCurr":500.000000, "JumpRangeMax":500.000000, 
            // "PendingDecommission":false, "SpaceUsage":{ "TotalCapacity":25000, "Crew":6170, "Cargo":2853, "CargoSpaceReserved":2169, "ShipPacks":0, 
            // "ModulePacks":4453, "FreeSpace":9355 }, "Finance":{ "CarrierBalance":28458349715, "ReserveBalance":86981722, "AvailableBalance":28148606640, 
            // "ReservePercent":0, "TaxRate_shipyard":0, "TaxRate_rearm":100, "TaxRate_outfitting":100, "TaxRate_refuel":100, "TaxRate_repair":100 }, 
            // "Crew":[ { "CrewRole":"BlackMarket", "Activated":false }, { "CrewRole":"Captain", "Activated":true, "Enabled":true, "CrewName":"Vada Cannon" }, { "CrewRole":"Refuel", "Activated":true, "Enabled":true, "CrewName":"Donna Moon" }, { "CrewRole":"Repair", "Activated":true, "Enabled":true, "CrewName":"Darnell Grant" }, { "CrewRole":"Rearm", "Activated":true, "Enabled":true, "CrewName":"Eiza York" }, { "CrewRole":"Commodities", "Activated":true, "Enabled":true, "CrewName":"Jewel King" }, { "CrewRole":"VoucherRedemption", "Activated":true, "Enabled":true, "CrewName":"Ezra Ramirez" }, { "CrewRole":"Exploration", "Activated":true, "Enabled":true, "CrewName":"Kasey Callahan" }, { "CrewRole":"Shipyard", "Activated":true, "Enabled":true, "CrewName":"Abby Cooke" }, { "CrewRole":"Outfitting", "Activated":true, "Enabled":true, "CrewName":"Jayne Callahan" }, { "CrewRole":"CarrierFuel", "Activated":true, "Enabled":true, "CrewName":"Abraham Strickland" }, { "CrewRole":"VistaGenomics", "Activated":true, "Enabled":true, "CrewName":"Melinda Reilly" }, { "CrewRole":"PioneerSupplies", "Activated":false }, { "CrewRole":"Bartender", "Activated":true, "Enabled":true, "CrewName":"Dean Barlow" } ], 
            // "ShipPacks":[  ], "ModulePacks":[ { "PackTheme":"VehicleSupport", "PackTier":1 }, { "PackTheme":"Storage", "PackTier":2 }, { "PackTheme":"Limpets", "PackTier":1 }, { "PackTheme":"Sensors", "PackTier":3 }, { "PackTheme":"Mining Tools", "PackTier":3 }, { "PackTheme":"Mining Utilities", "PackTier":2 }, { "PackTheme":"ShipUtilities", "PackTier":2 }, { "PackTheme":"TravelEnhancements", "PackTier":3 } ] }
            thread::spawn(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async move {
                        let mut services = String::new();
                        for entry in 0..json["Crew"].len() {
                            if json["Crew"][entry]["Activated"].as_bool().unwrap_or(false) {
                                if !services.is_empty(){
                                    services.push(',');
                                }
                                services.push_str(json["Crew"][entry]["CrewRole"].to_string().as_str());
                            }
                        }
                        
                        let contract = get_contract().await;
                        let function_call: FunctionCall<
                            Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
                            SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                            (),
                        > = contract.register_carrier(
                            json["CarrierID"].as_u64().unwrap(),
                            json["Name"].as_str().unwrap().to_string(),
                            json["Callsign"].as_str().unwrap().to_string(),
                            services,
                            json["DockingAccess"].as_str().unwrap().to_string(),
                            json["AllowNotorious"].as_bool().unwrap(),
                            DateTime::parse_from_rfc3339(
                                json["timestamp"].as_str().unwrap(),
                            )
                                .unwrap()
                                .timestamp()
                                .into()
                        );
                        //execute_send(function_call).await;
                        execute_send_repeatable(function_call).await;
                    });
            });
        }
        "CarrierTradeOrder" => {}
        "CarrierFinance" => {}
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
        "SupercruiseDestinationDrop" => {
            //TODO May update carrier name
            //{ "timestamp":"2024-04-02T21:21:39Z", "event":"SupercruiseDestinationDrop", "Type":"FUXBAU Q2K-BHB", "Threat":0, "MarketID":3704402432 }
        }
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
        _ => {
            // {"commodities":[{
            // "buyPrice":0,
            // "demand":27783,
            // "demandBracket":3,
            // "meanPrice":1485,
            // "name":"advancedmedicines",
            // "sellPrice":1730,
            // "stock":0,
            // "stockBracket":0},
            // {"buyPrice":0,"demand":186,"demandBracket":3,"meanPrice":3105,"name":"agronomictreatment","sellPrice":6444,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":293228,"demandBracket":2,"meanPrice":356,"name":"algae","sellPrice":625,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":1426017,"demandBracket":2,"meanPrice":551,"name":"aluminium","sellPrice":749,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":22047,"demandBracket":2,"meanPrice":1539,"name":"animalmeat","sellPrice":1881,"stock":0,"stockBracket":0},{"buyPrice":302,"demand":1,"demandBracket":0,"meanPrice":571,"name":"atmosphericextractors","sellPrice":280,"stock":118516,"stockBracket":3},{"buyPrice":0,"demand":59545,"demandBracket":3,"meanPrice":3826,"name":"autofabricators","sellPrice":4682,"stock":0,"stockBracket":0},{"buyPrice":523,"demand":1,"demandBracket":0,"meanPrice":493,"name":"basicmedicines","sellPrice":472,"statusFlags":["Narrative"],"stock":74,"stockBracket":1},{"buyPrice":0,"demand":140193,"demandBracket":2,"meanPrice":430,"name":"beer","sellPrice":658,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":110895,"demandBracket":2,"meanPrice":8242,"name":"beryllium","sellPrice":8945,"stock":0,"stockBracket":0},{"buyPrice":67,"demand":1,"demandBracket":0,"meanPrice":358,"name":"biowaste","sellPrice":40,"stock":12000,"stockBracket":2},{"buyPrice":0,"demand":1942,"demandBracket":3,"meanPrice":779,"name":"bootlegliquor","sellPrice":638,"stock":0,"stockBracket":0},{"buyPrice":1805,"demand":1,"demandBracket":0,"meanPrice":2311,"name":"buildingfabricators","sellPrice":1750,"stock":407734,"stockBracket":3},{"buyPrice":0,"demand":922972,"demandBracket":2,"meanPrice":415,"name":"ceramiccomposites","sellPrice":637,"stock":0,"stockBracket":0},{"buyPrice":430,"demand":1,"demandBracket":0,"meanPrice":546,"name":"clothing","sellPrice":389,"stock":8835,"stockBracket":2},{"buyPrice":0,"demand":61942,"demandBracket":2,"meanPrice":5987,"name":"cmmcomposite","sellPrice":6648,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":10467,"demandBracket":1,"meanPrice":3761,"name":"cobalt","sellPrice":7402,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":7269,"demandBracket":3,"meanPrice":1498,"name":"coffee","sellPrice":1895,"stock":0,"stockBracket":0},{"buyPrice":405,"demand":1,"demandBracket":0,"meanPrice":775,"name":"computercomponents","sellPrice":377,"stock":6790,"stockBracket":3},{"buyPrice":0,"demand":450154,"demandBracket":2,"meanPrice":709,"name":"conductivefabrics","sellPrice":1896,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":12032,"demandBracket":2,"meanPrice":6691,"name":"consumertechnology","sellPrice":7497,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":211861,"demandBracket":3,"meanPrice":1886,"name":"coolinghoses","sellPrice":1810,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":1059811,"demandBracket":2,"meanPrice":689,"name":"copper","sellPrice":925,"stock":0,"stockBracket":0},{"buyPrice":2002,"demand":1,"demandBracket":0,"meanPrice":2230,"name":"cropharvesters","sellPrice":1941,"stock":73490,"stockBracket":3},{"buyPrice":0,"demand":1082,"demandBracket":3,"meanPrice":16966,"name":"damagedescapepod","sellPrice":17148,"stock":0,"stockBracket":0},{"buyPrice":480,"demand":1,"demandBracket":0,"meanPrice":740,"name":"domesticappliances","sellPrice":447,"stock":5322,"stockBracket":2},{"buyPrice":0,"demand":24872,"demandBracket":2,"meanPrice":2368,"name":"emergencypowercells","sellPrice":2825,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":64934,"demandBracket":2,"meanPrice":649,"name":"fish","sellPrice":901,"stock":0,"stockBracket":0},{"buyPrice":54,"demand":1,"demandBracket":0,"meanPrice":266,"name":"foodcartridges","sellPrice":35,"stock":790979,"stockBracket":3},{"buyPrice":0,"demand":25943,"demandBracket":3,"meanPrice":509,"name":"fruitandvegetables","sellPrice":813,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":129261,"demandBracket":2,"meanPrice":5202,"name":"gallium","sellPrice":5610,"stock":0,"stockBracket":0},{"buyPrice":1465,"demand":1,"demandBracket":0,"meanPrice":1886,"name":"geologicalequipment","sellPrice":1419,"stock":8273,"stockBracket":3},{"buyPrice":0,"demand":20142,"demandBracket":2,"meanPrice":47609,"name":"gold","sellPrice":56031,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":142819,"demandBracket":2,"meanPrice":411,"name":"grain","sellPrice":667,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":422868,"demandBracket":2,"meanPrice":570,"name":"hazardousenvironmentsuits","sellPrice":768,"stock":0,"stockBracket":0},{"buyPrice":1465,"demand":1,"demandBracket":0,"meanPrice":1923,"name":"hnshockmount","sellPrice":1419,"stock":99648,"stockBracket":3},{"buyPrice":0,"demand":217,"demandBracket":3,"meanPrice":34605,"name":"hostage","sellPrice":34428,"stock":0,"stockBracket":0},{"buyPrice":85,"demand":1,"demandBracket":0,"meanPrice":113,"name":"hydrogenfuel","sellPrice":80,"stock":376271,"stockBracket":3},{"buyPrice":0,"demand":293899,"demandBracket":2,"meanPrice":3161,"name":"hydrogenperoxide","sellPrice":3012,"stock":0,"stockBracket":0},{"buyPrice":16683,"demand":1,"demandBracket":0,"meanPrice":17493,"name":"imperialslaves","sellPrice":16419,"stock":32257,"stockBracket":2},{"buyPrice":0,"demand":40718,"demandBracket":2,"meanPrice":5844,"name":"indium","sellPrice":6052,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":39465,"demandBracket":3,"meanPrice":10724,"name":"insulatingmembrane","sellPrice":11615,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":1163015,"demandBracket":2,"meanPrice":435,"name":"leather","sellPrice":660,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":510832,"demandBracket":2,"meanPrice":1473,"name":"liquidoxygen","sellPrice":2086,"stock":0,"stockBracket":0},{"buyPrice":616,"demand":1,"demandBracket":0,"meanPrice":879,"name":"liquor","sellPrice":584,"stock":160,"stockBracket":2},{"buyPrice":0,"demand":369612,"demandBracket":2,"meanPrice":1771,"name":"lithium","sellPrice":2078,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":1855,"demandBracket":3,"meanPrice":106382,"name":"lowtemperaturediamond","sellPrice":202708,"stock":0,"stockBracket":0},{"buyPrice":3607,"demand":1,"demandBracket":0,"meanPrice":4136,"name":"marinesupplies","sellPrice":3501,"stock":46491,"stockBracket":3},{"buyPrice":0,"demand":16607,"demandBracket":3,"meanPrice":5590,"name":"microcontrollers","sellPrice":6266,"stock":0,"stockBracket":0},{"buyPrice":771,"demand":1,"demandBracket":0,"meanPrice":801,"name":"mineralextractors","sellPrice":731,"stock":143115,"stockBracket":2},{"buyPrice":0,"demand":557830,"demandBracket":3,"meanPrice":687,"name":"naturalfabrics","sellPrice":913,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":3978,"demandBracket":2,"meanPrice":1943,"name":"nonlethalweapons","sellPrice":2419,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":217,"demandBracket":3,"meanPrice":30364,"name":"occupiedcryopod","sellPrice":30184,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":14332,"demandBracket":3,"meanPrice":45210,"name":"osmium","sellPrice":170697,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":1763,"demandBracket":3,"meanPrice":53032,"name":"painite","sellPrice":100940,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":16720,"demandBracket":2,"meanPrice":50639,"name":"palladium","sellPrice":59178,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":4674,"demandBracket":3,"meanPrice":6790,"name":"performanceenhancers","sellPrice":7517,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":217,"demandBracket":3,"meanPrice":9544,"name":"personaleffects","sellPrice":9439,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":1246,"demandBracket":2,"meanPrice":58272,"name":"platinum","sellPrice":227269,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":1360424,"demandBracket":2,"meanPrice":376,"name":"polymers","sellPrice":584,"stock":0,"stockBracket":0},{"buyPrice":2146,"demand":1,"demandBracket":0,"meanPrice":2466,"name":"powergenerators","sellPrice":2081,"stock":4106,"stockBracket":2},{"buyPrice":0,"demand":125659,"demandBracket":2,"meanPrice":8620,"name":"praseodymium","sellPrice":8500,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":6048,"demandBracket":2,"meanPrice":6751,"name":"progenitorcells","sellPrice":7516,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":6019,"demandBracket":3,"meanPrice":2224,"name":"reactivearmour","sellPrice":2681,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":142476,"demandBracket":2,"meanPrice":2019,"name":"robotics","sellPrice":2736,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":79966,"demandBracket":3,"meanPrice":25853,"name":"samarium","sellPrice":25625,"stock":0,"stockBracket":0},{"buyPrice":97,"demand":1,"demandBracket":0,"meanPrice":300,"name":"scrap","sellPrice":80,"stock":222939,"stockBracket":3},{"buyPrice":0,"demand":1349309,"demandBracket":2,"meanPrice":1136,"name":"semiconductors","sellPrice":1448,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":35655,"demandBracket":2,"meanPrice":37223,"name":"silver","sellPrice":48944,"stock":0,"stockBracket":0},{"buyPrice":754,"demand":1,"demandBracket":0,"meanPrice":1119,"name":"skimercomponents","sellPrice":718,"stock":10398,"stockBracket":3},{"buyPrice":0,"demand":8198,"demandBracket":1,"meanPrice":6678,"name":"superconductors","sellPrice":6561,"stock":0,"stockBracket":0},{"buyPrice":809,"demand":1,"demandBracket":0,"meanPrice":684,"name":"survivalequipment","sellPrice":754,"stock":5144,"stockBracket":3},{"buyPrice":0,"demand":1305546,"demandBracket":2,"meanPrice":416,"name":"syntheticfabrics","sellPrice":631,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":16884,"demandBracket":2,"meanPrice":440,"name":"syntheticmeat","sellPrice":735,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":77804,"demandBracket":2,"meanPrice":4043,"name":"tantalum","sellPrice":4381,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":12258,"demandBracket":2,"meanPrice":1695,"name":"tea","sellPrice":1726,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":930,"demandBracket":3,"meanPrice":148147,"name":"thargoidpod","sellPrice":149716,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":930,"demandBracket":3,"meanPrice":70981,"name":"thargoidtissuesampletype6","sellPrice":72064,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":930,"demandBracket":3,"meanPrice":496635,"name":"thargoidtissuesampletype9a","sellPrice":499599,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":930,"demandBracket":3,"meanPrice":271074,"name":"thargoidtissuesampletype9b","sellPrice":273224,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":930,"demandBracket":3,"meanPrice":138542,"name":"thargoidtissuesampletype9c","sellPrice":140058,"stock":0,"stockBracket":0},{"buyPrice":3135,"demand":1,"demandBracket":0,"meanPrice":3761,"name":"thermalcoolingunits","sellPrice":3043,"stock":20444,"stockBracket":3},{"buyPrice":0,"demand":488199,"demandBracket":2,"meanPrice":1208,"name":"titanium","sellPrice":1436,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":11094,"demandBracket":2,"meanPrice":5324,"name":"tobacco","sellPrice":5537,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":7171,"demandBracket":3,"meanPrice":51707,"name":"tritium","sellPrice":57597,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":930,"demandBracket":3,"meanPrice":3900,"name":"unocuppiedescapepod","sellPrice":4232,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":247271,"demandBracket":2,"meanPrice":2826,"name":"uranium","sellPrice":3197,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":217,"demandBracket":3,"meanPrice":31063,"name":"usscargoblackbox","sellPrice":30857,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":160169,"demandBracket":3,"meanPrice":279,"name":"water","sellPrice":621,"stock":0,"stockBracket":0},{"buyPrice":214,"demand":1,"demandBracket":0,"meanPrice":484,"name":"waterpurifiers","sellPrice":193,"stock":1168540,"stockBracket":3},{"buyPrice":0,"demand":81700,"demandBracket":2,"meanPrice":507,"name":"wine","sellPrice":740,"stock":0,"stockBracket":0},{"buyPrice":0,"demand":2164,"demandBracket":3,"meanPrice":9034,"name":"wreckagecomponents","sellPrice":9146,"stock":0,"stockBracket":0}],"economies":[{"name":"Industrial","proportion":1}],
            // "horizons":true,
            // "marketId":3224312064,"odyssey":true,"prohibited":["BasicNarcotics","BattleWeapons","CombatStabilisers","Landmines","OnionHeadC","PersonalWeapons","Slaves"],
            // "stationName":"Schmidt Orbital","systemName":"Murare","timestamp":"2024-04-04T21:11:37Z"}

            // {"horizons":true,"marketId":3224312064,"modules":["Hpt_advancedtorppylon_fixed_large","Hpt_advancedtorppylon_fixed_medium","Hpt_advancedtorppylon_fixed_small","Hpt_basicmissilerack_fixed_medium","Hpt_basicmissilerack_fixed_small","Hpt_beamlaser_fixed_large","Hpt_beamlaser_gimbal_huge","Hpt_beamlaser_gimbal_large","Hpt_beamlaser_gimbal_medium","Hpt_beamlaser_gimbal_small","Hpt_beamlaser_turret_medium","Hpt_cannon_fixed_huge","Hpt_cannon_fixed_large","Hpt_cannon_gimbal_huge","Hpt_cannon_gimbal_large","Hpt_cannon_turret_large","Hpt_cannon_turret_small","Hpt_cargoscanner_size0_class1","Hpt_cargoscanner_size0_class4","Hpt_cloudscanner_size0_class1","Hpt_cloudscanner_size0_class5","Hpt_crimescanner_size0_class2","Hpt_crimescanner_size0_class5","Hpt_dumbfiremissilerack_fixed_large","Hpt_dumbfiremissilerack_fixed_medium","Hpt_dumbfiremissilerack_fixed_small","Hpt_electroniccountermeasure_tiny","Hpt_heatsinklauncher_turret_tiny","Hpt_minelauncher_fixed_small","Hpt_minelauncher_fixed_small_impulse","Hpt_mining_abrblstr_fixed_small","Hpt_mining_abrblstr_turret_small","Hpt_mining_seismchrgwarhd_turret_medium","Hpt_mining_subsurfdispmisle_fixed_medium","Hpt_mining_subsurfdispmisle_fixed_small","Hpt_mining_subsurfdispmisle_turret_small","Hpt_mininglaser_fixed_medium","Hpt_mininglaser_fixed_small","Hpt_mrascanner_size0_class1","Hpt_mrascanner_size0_class2","Hpt_mrascanner_size0_class3","Hpt_mrascanner_size0_class4","Hpt_multicannon_fixed_huge","Hpt_multicannon_gimbal_huge","Hpt_multicannon_gimbal_large","Hpt_multicannon_turret_large","Hpt_plasmaaccelerator_fixed_large","Hpt_plasmaaccelerator_fixed_medium","Hpt_plasmapointdefence_turret_tiny","Hpt_pulselaser_fixed_large","Hpt_pulselaser_fixed_medium","Hpt_pulselaser_fixed_small","Hpt_pulselaser_gimbal_medium","Hpt_pulselaser_gimbal_small","Hpt_pulselaser_turret_small","Hpt_pulselaserburst_fixed_large","Hpt_pulselaserburst_fixed_medium","Hpt_pulselaserburst_fixed_small","Hpt_pulselaserburst_gimbal_large","Hpt_pulselaserburst_gimbal_small","Hpt_pulselaserburst_turret_medium","Hpt_pulselaserburst_turret_small","Hpt_railgun_fixed_medium","Hpt_railgun_fixed_small","Hpt_slugshot_gimbal_large","Hpt_slugshot_gimbal_small","Hpt_slugshot_turret_large","Hpt_slugshot_turret_medium","Hpt_slugshot_turret_small","Int_buggybay_size2_class1","Int_buggybay_size4_class1","Int_buggybay_size4_class2","Int_buggybay_size6_class1","Int_buggybay_size6_class2","Int_cargorack_size1_class1","Int_cargorack_size2_class1","Int_cargorack_size3_class1","Int_cargorack_size4_class1","Int_cargorack_size5_class1","Int_cargorack_size6_class1","Int_cargorack_size7_class1","Int_cargorack_size8_class1","Int_detailedsurfacescanner_tiny","Int_dockingcomputer_advanced","Int_dronecontrol_collection_size1_class4","Int_dronecontrol_collection_size1_class5","Int_dronecontrol_collection_size3_class5","Int_dronecontrol_collection_size5_class3","Int_dronecontrol_collection_size5_class4","Int_dronecontrol_collection_size7_class2","Int_dronecontrol_collection_size7_class3","Int_dronecontrol_collection_size7_class4","Int_dronecontrol_fueltransfer_size1_class3","Int_dronecontrol_fueltransfer_size1_class4","Int_dronecontrol_fueltransfer_size1_class5","Int_dronecontrol_fueltransfer_size3_class2","Int_dronecontrol_fueltransfer_size3_class3","Int_dronecontrol_fueltransfer_size3_class4","Int_dronecontrol_fueltransfer_size3_class5","Int_dronecontrol_fueltransfer_size5_class1","Int_dronecontrol_fueltransfer_size5_class2","Int_dronecontrol_fueltransfer_size5_class4","Int_dronecontrol_fueltransfer_size5_class5","Int_dronecontrol_fueltransfer_size7_class1","Int_dronecontrol_fueltransfer_size7_class3","Int_dronecontrol_fueltransfer_size7_class5","Int_dronecontrol_prospector_size1_class2","Int_dronecontrol_prospector_size1_class3","Int_dronecontrol_prospector_size1_class4","Int_dronecontrol_prospector_size1_class5","Int_dronecontrol_prospector_size3_class1","Int_dronecontrol_prospector_size3_class2","Int_dronecontrol_prospector_size3_class3","Int_dronecontrol_prospector_size3_class4","Int_dronecontrol_prospector_size3_class5","Int_dronecontrol_prospector_size5_class1","Int_dronecontrol_prospector_size5_class2","Int_dronecontrol_prospector_size5_class3","Int_dronecontrol_prospector_size5_class4","Int_dronecontrol_prospector_size7_class1","Int_dronecontrol_prospector_size7_class2","Int_dronecontrol_prospector_size7_class3","Int_dronecontrol_prospector_size7_class4","Int_dronecontrol_prospector_size7_class5","Int_dronecontrol_recon_size7_class1","Int_dronecontrol_repair_size1_class3","Int_dronecontrol_repair_size1_class4","Int_dronecontrol_repair_size1_class5","Int_dronecontrol_repair_size3_class3","Int_dronecontrol_repair_size3_class5","Int_dronecontrol_repair_size5_class1","Int_dronecontrol_repair_size5_class2","Int_dronecontrol_repair_size5_class4","Int_dronecontrol_repair_size5_class5","Int_dronecontrol_repair_size7_class1","Int_dronecontrol_repair_size7_class3","Int_dronecontrol_resourcesiphon_size1_class1","Int_dronecontrol_resourcesiphon_size1_class2","Int_dronecontrol_resourcesiphon_size1_class3","Int_dronecontrol_resourcesiphon_size1_class4","Int_dronecontrol_resourcesiphon_size1_class5","Int_dronecontrol_resourcesiphon_size3_class1","Int_dronecontrol_resourcesiphon_size3_class2","Int_dronecontrol_resourcesiphon_size3_class3","Int_dronecontrol_resourcesiphon_size3_class4","Int_dronecontrol_resourcesiphon_size3_class5","Int_dronecontrol_resourcesiphon_size5_class1","Int_dronecontrol_resourcesiphon_size5_class2","Int_dronecontrol_resourcesiphon_size5_class3","Int_dronecontrol_resourcesiphon_size5_class4","Int_dronecontrol_resourcesiphon_size7_class1","Int_dronecontrol_resourcesiphon_size7_class2","Int_dronecontrol_resourcesiphon_size7_class3","Int_dronecontrol_resourcesiphon_size7_class4","Int_engine_size2_class1","Int_engine_size2_class2","Int_engine_size2_class3","Int_engine_size2_class4","Int_engine_size3_class1","Int_engine_size3_class2","Int_engine_size3_class3","Int_engine_size3_class4","Int_engine_size3_class5","Int_engine_size7_class3","Int_engine_size7_class4","Int_engine_size7_class5","Int_engine_size8_class3","Int_fsdinterdictor_size1_class1","Int_fsdinterdictor_size1_class2","Int_fsdinterdictor_size1_class3","Int_fsdinterdictor_size1_class4","Int_fsdinterdictor_size2_class1","Int_fsdinterdictor_size2_class2","Int_fsdinterdictor_size2_class3","Int_fsdinterdictor_size2_class4","Int_fsdinterdictor_size2_class5","Int_fuelscoop_size1_class1","Int_fuelscoop_size1_class3","Int_fuelscoop_size1_class4","Int_fuelscoop_size1_class5","Int_fuelscoop_size2_class2","Int_fuelscoop_size2_class3","Int_fuelscoop_size2_class4","Int_fuelscoop_size2_class5","Int_fuelscoop_size3_class2","Int_fuelscoop_size3_class3","Int_fuelscoop_size3_class4","Int_fuelscoop_size3_class5","Int_fuelscoop_size7_class1","Int_fuelscoop_size7_class2","Int_fuelscoop_size7_class3","Int_fuelscoop_size7_class4","Int_fuelscoop_size8_class1","Int_fuelscoop_size8_class2","Int_fuelscoop_size8_class4","Int_fueltank_size6_class3","Int_fueltank_size7_class3","Int_fueltank_size8_class3","Int_hullreinforcement_size1_class2","Int_hullreinforcement_size2_class2","Int_hullreinforcement_size3_class2","Int_hullreinforcement_size4_class2","Int_hullreinforcement_size5_class2","Int_hyperdrive_size2_class1","Int_hyperdrive_size2_class2","Int_hyperdrive_size2_class3","Int_hyperdrive_size2_class4","Int_hyperdrive_size2_class5","Int_hyperdrive_size3_class1","Int_hyperdrive_size3_class2","Int_hyperdrive_size3_class3","Int_hyperdrive_size3_class5","Int_hyperdrive_size4_class1","Int_hyperdrive_size4_class2","Int_hyperdrive_size4_class3","Int_hyperdrive_size4_class4","Int_hyperdrive_size4_class5","Int_hyperdrive_size5_class1","Int_hyperdrive_size5_class2","Int_hyperdrive_size5_class3","Int_hyperdrive_size5_class4","Int_hyperdrive_size5_class5","Int_hyperdrive_size6_class1","Int_hyperdrive_size6_class2","Int_hyperdrive_size6_class3","Int_hyperdrive_size7_class1","Int_hyperdrive_size7_class2","Int_hyperdrive_size7_class3","Int_lifesupport_size1_class2","Int_lifesupport_size1_class3","Int_lifesupport_size1_class4","Int_lifesupport_size2_class2","Int_lifesupport_size2_class3","Int_lifesupport_size2_class4","Int_lifesupport_size2_class5","Int_lifesupport_size3_class2","Int_lifesupport_size3_class3","Int_lifesupport_size3_class4","Int_lifesupport_size3_class5","Int_lifesupport_size7_class4","Int_lifesupport_size7_class5","Int_lifesupport_size8_class2","Int_lifesupport_size8_class3","Int_modulereinforcement_size1_class2","Int_modulereinforcement_size2_class2","Int_modulereinforcement_size3_class2","Int_modulereinforcement_size4_class2","Int_modulereinforcement_size5_class2","Int_multidronecontrol_mining_size3_class3","Int_multidronecontrol_operations_size3_class3","Int_multidronecontrol_operations_size3_class4","Int_multidronecontrol_rescue_size3_class3","Int_powerdistributor_size1_class1","Int_powerdistributor_size1_class2","Int_powerdistributor_size1_class3","Int_powerdistributor_size1_class4","Int_powerdistributor_size2_class1","Int_powerdistributor_size2_class2","Int_powerdistributor_size2_class3","Int_powerdistributor_size2_class4","Int_powerdistributor_size2_class5","Int_powerdistributor_size3_class1","Int_powerdistributor_size3_class2","Int_powerdistributor_size3_class3","Int_powerdistributor_size3_class4","Int_powerdistributor_size3_class5","Int_powerdistributor_size4_class1","Int_powerdistributor_size4_class2","Int_powerdistributor_size4_class3","Int_powerdistributor_size5_class1","Int_powerdistributor_size5_class2","Int_powerdistributor_size5_class3","Int_powerdistributor_size5_class4","Int_powerdistributor_size6_class1","Int_powerdistributor_size6_class2","Int_powerdistributor_size6_class3","Int_powerdistributor_size6_class4","Int_powerplant_size2_class1","Int_powerplant_size2_class2","Int_powerplant_size2_class3","Int_powerplant_size2_class4","Int_powerplant_size2_class5","Int_powerplant_size3_class1","Int_powerplant_size3_class2","Int_powerplant_size3_class3","Int_powerplant_size3_class4","Int_powerplant_size3_class5","Int_powerplant_size4_class1","Int_powerplant_size4_class2","Int_powerplant_size4_class3","Int_powerplant_size4_class4","Int_powerplant_size4_class5","Int_powerplant_size5_class1","Int_powerplant_size5_class2","Int_powerplant_size5_class3","Int_powerplant_size5_class4","Int_powerplant_size5_class5","Int_powerplant_size6_class1","Int_powerplant_size6_class2","Int_powerplant_size6_class3","Int_powerplant_size6_class4","Int_powerplant_size7_class2","Int_powerplant_size7_class3","Int_powerplant_size7_class4","Int_powerplant_size7_class5","Int_powerplant_size8_class1","Int_powerplant_size8_class3","Int_refinery_size1_class1","Int_refinery_size1_class2","Int_refinery_size1_class3","Int_refinery_size1_class4","Int_refinery_size1_class5","Int_refinery_size2_class1","Int_refinery_size2_class2","Int_refinery_size2_class3","Int_refinery_size2_class5","Int_refinery_size3_class2","Int_refinery_size3_class3","Int_refinery_size3_class4","Int_refinery_size3_class5","Int_refinery_size4_class1","Int_refinery_size4_class2","Int_refinery_size4_class3","Int_refinery_size4_class4","Int_refinery_size4_class5","Int_repairer_size2_class4","Int_repairer_size3_class3","Int_repairer_size4_class5","Int_repairer_size5_class2","Int_repairer_size5_class3","Int_repairer_size5_class4","Int_repairer_size5_class5","Int_repairer_size6_class2","Int_repairer_size6_class3","Int_repairer_size7_class1","Int_repairer_size7_class2","Int_repairer_size7_class3","Int_repairer_size7_class4","Int_repairer_size8_class1","Int_repairer_size8_class2","Int_repairer_size8_class4","Int_sensors_size1_class1","Int_sensors_size1_class2","Int_sensors_size1_class3","Int_sensors_size1_class4","Int_sensors_size1_class5","Int_sensors_size2_class1","Int_sensors_size2_class2","Int_sensors_size2_class3","Int_sensors_size2_class4","Int_sensors_size2_class5","Int_sensors_size3_class1","Int_sensors_size3_class2","Int_sensors_size3_class3","Int_sensors_size3_class4","Int_sensors_size3_class5","Int_sensors_size7_class1","Int_sensors_size7_class2","Int_sensors_size7_class3","Int_sensors_size7_class5","Int_sensors_size8_class1","Int_sensors_size8_class2","Int_sensors_size8_class3","Int_sensors_size8_class4","Int_shieldcellbank_size1_class1","Int_shieldcellbank_size1_class2","Int_shieldcellbank_size1_class3","Int_shieldcellbank_size1_class4","Int_shieldcellbank_size1_class5","Int_shieldcellbank_size2_class2","Int_shieldcellbank_size2_class3","Int_shieldcellbank_size2_class4","Int_shieldcellbank_size3_class1","Int_shieldcellbank_size3_class2","Int_shieldcellbank_size3_class3","Int_shieldcellbank_size3_class4","Int_shieldcellbank_size3_class5","Int_shieldcellbank_size4_class1","Int_shieldcellbank_size4_class2","Int_shieldcellbank_size4_class3","Int_shieldcellbank_size4_class5","Int_shieldcellbank_size5_class1","Int_shieldcellbank_size5_class2","Int_shieldcellbank_size5_class4","Int_shieldcellbank_size5_class5","Int_shieldcellbank_size6_class1","Int_shieldcellbank_size6_class2","Int_shieldcellbank_size6_class3","Int_shieldcellbank_size6_class4","Int_shieldcellbank_size6_class5","Int_shieldcellbank_size7_class4","Int_shieldcellbank_size8_class1","Int_shieldcellbank_size8_class5","Int_shieldgenerator_size1_class3_fast","Int_shieldgenerator_size2_class1","Int_shieldgenerator_size2_class2","Int_shieldgenerator_size2_class3","Int_shieldgenerator_size2_class3_fast","Int_shieldgenerator_size2_class4","Int_shieldgenerator_size2_class5","Int_shieldgenerator_size3_class1","Int_shieldgenerator_size3_class2","Int_shieldgenerator_size3_class3","Int_shieldgenerator_size3_class3_fast","Int_shieldgenerator_size3_class4","Int_shieldgenerator_size7_class4","Int_shieldgenerator_size7_class5","Int_shieldgenerator_size8_class3","Int_shieldgenerator_size8_class3_fast","Int_shieldgenerator_size8_class4","Int_supercruiseassist","belugaliner_Armour_grade1","belugaliner_Armour_grade2","belugaliner_Armour_grade3","belugaliner_Armour_mirrored","belugaliner_Armour_reactive","cobramkiii_Armour_grade1","cobramkiii_Armour_grade2","cobramkiii_Armour_grade3","cobramkiii_Armour_mirrored","cobramkiii_Armour_reactive","dolphin_Armour_grade1","dolphin_Armour_grade2","dolphin_Armour_grade3","dolphin_Armour_mirrored","dolphin_Armour_reactive","empire_courier_Armour_grade1","empire_courier_Armour_grade2","empire_courier_Armour_grade3","empire_courier_Armour_mirrored","empire_courier_Armour_reactive","empire_eagle_Armour_grade1","empire_eagle_Armour_grade2","empire_eagle_Armour_grade3","empire_eagle_Armour_mirrored","empire_eagle_Armour_reactive","empire_trader_Armour_grade1","empire_trader_Armour_grade2","empire_trader_Armour_grade3","empire_trader_Armour_mirrored","empire_trader_Armour_reactive","hauler_Armour_grade1","hauler_Armour_grade2","hauler_Armour_grade3","hauler_Armour_mirrored","hauler_Armour_reactive","orca_Armour_grade1","orca_Armour_grade2","orca_Armour_grade3","orca_Armour_mirrored","orca_Armour_reactive","sidewinder_Armour_grade1","sidewinder_Armour_grade2","sidewinder_Armour_grade3","sidewinder_Armour_mirrored","sidewinder_Armour_reactive","type6_Armour_grade1","type6_Armour_grade2","type6_Armour_grade3","type6_Armour_mirrored","type6_Armour_reactive","type7_Armour_grade1","type7_Armour_grade2","type7_Armour_grade3","type7_Armour_mirrored","type7_Armour_reactive","viper_mkiv_Armour_grade1","viper_mkiv_Armour_grade2","viper_mkiv_Armour_grade3","viper_mkiv_Armour_mirrored","viper_mkiv_Armour_reactive"],"odyssey":true,"stationName":"Schmidt Orbital","systemName":"Murare","timestamp":"2024-04-04T21:11:37Z"}
            // MiddlewareError: (code: -32000, message: invalid transaction nonce: got 571238, want 571239, data: None)
            // {"horizons":true,"marketId":3224312064,"odyssey":true,"ships":["belugaliner","cobramkiii","dolphin","empire_courier","empire_eagle","empire_trader","hauler","orca","sidewinder","type6","type7","viper_mkiv"],"stationName":"Schmidt Orbital","systemName":"Murare","timestamp":"2024-04-04T21:11:37Z"}
            thread::spawn(|| {
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async move {
                        let contract = get_contract().await;

                        if !json["commodities"].is_empty() {
                            let market_id = json["marketId"].as_u64().unwrap();
                            let size = json["commodities"].len();
                            for i in 0..size {
                                let commoditiy = &json["commodities"][i];
                                let function_call: FunctionCall<
                                    Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
                                    SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                                    (),
                                > = contract.register_commodity_listening(
                                    market_id,
                                    commoditiy["name"].as_str().unwrap().to_ascii_lowercase(),
                                    edcas_contract::CommodityListening{
                                        buy_price: commoditiy["buyPrice"].as_u32().unwrap_or(0),
                                        sell_price: commoditiy["sellPrice"].as_u32().unwrap_or(0),
                                        mean_price: commoditiy["meanPrice"].as_u32().unwrap_or(0),
                                        stock: commoditiy["stock"].as_u32().unwrap_or(0),
                                        demand: commoditiy["demand"].as_u32().unwrap_or(0),
                                        stock_bracket: commoditiy["stockBracket"].as_u32().unwrap_or(0),
                                        demand_bracket: commoditiy["demandBracket"].as_u32().unwrap_or(0),
                                    }
                                );
                                //execute_send(function_call).await;
                                execute_send_repeatable(function_call).await;
                            }
                        }
                    });
            });
        }
    }
}

fn process_jump(json: JsonValue) {
    thread::spawn(|| {
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
                    json["Population"].as_u64().unwrap_or(0),
                    DateTime::parse_from_rfc3339(
                        json["timestamp"].as_str().unwrap(),
                    )
                        .unwrap()
                        .timestamp()
                        .into()
                );
                //execute_send(function_call).await;
                execute_send_repeatable(function_call).await;
            });
    });
}

fn extract_planet_properties(json: &JsonValue) -> PlanetProperties {
    PlanetProperties {
        atmosphere: json["Atmosphere"].to_string(),
        class: json["PlanetClass"].to_string(),
        landable: json["Landable"].as_bool().unwrap_or(false),
        terraform_state: json["TerraformState"].to_string(),
        volcanism: json["Volcanism"].to_string(),
        tidal_lock: json["TidalLock"].as_bool().unwrap_or_else(|| panic!("Tidal Lock not parseable")),
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
            decimal: json["Eccentricity"].to_string().replace('.', "").parse().unwrap_or_else(|_| panic!("Eccentricity invalid parse")),
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
                tokio::time::sleep(Duration::from_secs(env::var("DURATION_TIMEOUT").unwrap_or("5".into()).parse().unwrap_or(5))).await;
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
                            eprintln!("JsonRpcClientError: {}",err);
                            Err(RepeatableError(err.to_string()))
                        }
                        ProviderError::EnsError(err) => {
                            eprintln!("EnsError: {}",err);
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::EnsNotOwned(err) => {
                            eprintln!("EnsNotOwned: {}",err);
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::SerdeJson(err) => {
                            eprintln!("SerdeJson: {}",err);
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::HexError(err) => {
                            eprintln!("HexError: {}",err);
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::HTTPError(err) => {
                            eprintln!("HTTPError: {}",err);
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::CustomError(err) => {
                            eprintln!("CustomError: {}",err);
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::UnsupportedRPC => {
                            eprintln!("UnsupportedRPC");
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::UnsupportedNodeClient => {
                            eprintln!("UnsupportedNodeClient");
                            Err(NonRepeatableError(err.to_string()))
                        }
                        ProviderError::SignerUnavailable => {
                            eprintln!("SignerUnavailable");
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
                    eprintln!("DecodingError: {}",err);
                    Err(NonRepeatableError(err.to_string()))
                }
                ContractError::AbiError(err) => {
                    eprintln!("AbiError: {}",err);
                    Err(NonRepeatableError(err.to_string()))
                }
                ContractError::DetokenizationError(err) => {
                    eprintln!("DetokenizationError: {}",err);
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
                    eprintln!("ConstructorError");
                    Err(NonRepeatableError(err.to_string()))
                }
                ContractError::ContractNotDeployed => {
                    eprintln!("ContractNotDeployed");
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
    let n = bytes.split_at(134/2).1;
    let n:&[u8] = n.split(|b|{
        *b == 0u8
    }).next().unwrap();
    String::from_utf8(n.to_vec()).unwrap()
}