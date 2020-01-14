#![feature(proc_macro_hygiene)]
#[macro_use]
extern crate hdk;
extern crate hdk_proc_macros;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate holochain_json_derive;

use hdk::holochain_core_types::{dna::entry_types::Sharing, entry::Entry};
use hdk::{entry_definition::ValidatingEntryType, error::ZomeApiResult};

use hdk::holochain_json_api::{error::JsonError, json::JsonString};

use hdk::holochain_persistence_api::cas::content::Address;

use hdk::prelude::LinkMatch;

use hdk_proc_macros::zome;
// see https://developer.holochain.org/api/0.0.38-alpha14/hdk/ for info on using the hdk library

// This is a sample zome that defines an entry type "MyEntry" that can be committed to the
// agent's chain via the exposed function create_my_entry

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Art {
    content: String,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Game {
    art: Address,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub enum Change {
    Add(u64, char),
    Delete(u64),
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct Edit {
    change: Vec<Change>,
}

#[derive(Serialize, Deserialize, Debug, DefaultJson, Clone)]
pub struct AcceptedEdit {
    edit: Address,
    order: u64,
}

#[zome]
mod art_game_zome {

    #[init]
    fn init() {
        Ok(())
    }

    #[validate_agent]
    pub fn validate_agent(validation_data: EntryValidationData<AgentId>) {
        Ok(())
    }

    #[entry_def]
    fn art_entry_def() -> ValidatingEntryType {
        entry!(
            name: "art",
            description: "This is some Art",
            sharing: Sharing::Public,
            validation_package: || {
                hdk::ValidationPackageDefinition::Entry
            },
            validation: | _validation_data: hdk::EntryValidationData<Art>| {
                Ok(())
            }
        )
    }

    #[entry_def]
    fn game_entry_def() -> ValidatingEntryType {
        entry!(
            name: "game",
            description:"This is an Art game",
            sharing:Sharing::Public,
            validation_package:||{
                hdk::ValidationPackageDefinition::Entry
            },
            validation:|_validation_data: hdk::EntryValidationData<Game>|{
                Ok(())
            },
                links:[
                    to!(
                        "art",
                        link_type:"current_art",
                        validation_package:||{
                            hdk::ValidationPackageDefinition::Entry
                        },
                        validation: |_validation_data: hdk::LinkValidationData|{
                            Ok(())
                        }
                    ),
                    to!(
                        "accepted_edit",
                        link_type:"accepted_edit",
                        validation_package:||{
                            hdk::ValidationPackageDefinition::Entry
                        },
                        validation:|_validation_data: hdk::LinkValidationData|{
                            Ok(())
                        }
                    )
                ]
        )
    }

    #[entry_def]
    fn anchor_entry_def() -> ValidatingEntryType {
        entry!(
            name: "anchor",
            description:"Anchor to all links",
            sharing: Sharing::Public,
            validation_package:||{
                hdk::ValidationPackageDefinition::Entry
            },
            validation:|_validation_data: hdk::EntryValidationData<String>|{
                Ok(())
            },
            links:[
                to!(
                    "game",
                    link_type:"has_game",
                    validation_package:||{
                        hdk::ValidationPackageDefinition::Entry
                    },
                    validation:|_validation_data: hdk::LinkValidationData|{
                        Ok(())
                    }
                )
            ]
        )
    }

    #[zome_fn("hc_public")]
    fn create_game(art: Art) -> ZomeApiResult<Address> {
        let art_entry = Entry::App("art".into(), art.into());

        let art_address = hdk::commit_entry(&art_entry)?;

        let game = Game {
            art: art_address.clone(),
        };

        let game_entry = Entry::App("game".into(), game.into());
        let game_address = hdk::commit_entry(&game_entry)?;

        let anchor_entry = Entry::App("anchor".into(), "game".into());
        let anchor_address = hdk::commit_entry(&anchor_entry)?;

        hdk::link_entries(&anchor_address, &game_address, "has_game", "")?;

        hdk::link_entries(&game_address, &art_address, "current_art", "")?;
        Ok(game_address)
    }

    #[zome_fn("hc_public")]
    fn submit_edit(game_address: Address, edit: Art) -> ZomeApiResult<Address> {
        let edit_entry = Entry::App("art".into(), edit.into());
        let edit_address = hdk::commit_entry(&edit_entry)?;

        let current_art_address = hdk::get_links(
            &game_address,
            LinkMatch::Exactly("current_art"),
            LinkMatch::Any,
        )?
        .addresses()
        .into_iter()
        .next();

        if let Some(current_art_address) = current_art_address {
            hdk::remove_link(&game_address, &current_art_address, "current_art", "")?;
        }
        hdk::link_entries(&game_address, &edit_address, "current_art", "")?;

        Ok(edit_address)
    }

    #[zome_fn("hc_public")]
    fn address_to_entry(address: Address) -> ZomeApiResult<Option<Entry>> {
        hdk::get_entry(&address)
    }

    #[zome_fn("hc_public")]
    fn get_games() -> ZomeApiResult<Vec<Game>> {
        //let anchor_entry = Entry::App("anchor".into(), "game".into());
        //    let anchor_address = get_anchor_address()?; //hdk::commit_entry(&anchor_entry)?;
        let anchor_address = get_anchor_address()?;

        hdk::utils::get_links_and_load_type(
            &anchor_address,
            LinkMatch::Exactly("has_game"),
            LinkMatch::Any,
        )
    }

    #[zome_fn("hc_public")]
    fn get_current_art(game: Address) -> ZomeApiResult<Option<Entry>> {
        hdk::get_links_and_load(&game, LinkMatch::Exactly("current_art"), LinkMatch::Any)
            .and_then(|current_art| current_art.into_iter().next().transpose())
    }

    //#[zome_fn("hc_public")]
    fn get_anchor_address() -> ZomeApiResult<Address> {
        let anchor_entry = Entry::App("anchor".into(), "game".into());
        let anchor_address = hdk::commit_entry(&anchor_entry)?;
        Ok(anchor_address)
    }

    #[zome_fn("hc_public")]
    fn get_anchor() -> ZomeApiResult<Address> {
        let anchor_entry = Entry::App("anchor".into(), "game".into());
        let anchor_address = hdk::commit_entry(&anchor_entry)?;
        Ok(anchor_address)
    }
}
