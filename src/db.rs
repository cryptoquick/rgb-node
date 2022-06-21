// RGB node providing smart contracts functionality for Bitcoin & Lightning.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2022 by LNP/BP Standards Association, Switzerland.
//
// You should have received a copy of the MIT License along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use amplify::Slice32;
use bitcoin::hashes::{sha256t, Hash};
use commit_verify::TaggedHash;
use internet2::addr::ServiceAddr;
use rgb::MergeReveal;
use strict_encoding::{StrictDecode, StrictEncode};

use crate::{DaemonError, LaunchError};

pub(crate) struct Db {
    pub(crate) store: store_rpc::Client,
}

impl Db {
    pub const SCHEMATA: &'static str = "schemata";
    pub const BUNDLES: &'static str = "bundles";
    pub const GENESIS: &'static str = "genesis";
    pub const TRANSITIONS: &'static str = "transitions";
    pub const ANCHORS: &'static str = "transitions";
    pub const EXTENSIONS: &'static str = "extensions";
    pub const ATTACHMENT_CHUNKS: &'static str = "chunks";
    pub const ATTACHMENT_INDEX: &'static str = "attachments";
    pub const ALU_LIBS: &'static str = "alu";

    pub fn with(store_endpoint: &ServiceAddr) -> Result<Db, LaunchError> {
        let mut store = store_rpc::Client::with(store_endpoint).map_err(LaunchError::from)?;

        for table in [
            Db::SCHEMATA,
            Db::BUNDLES,
            Db::GENESIS,
            Db::TRANSITIONS,
            Db::ANCHORS,
            Db::EXTENSIONS,
            Db::ATTACHMENT_CHUNKS,
            Db::ATTACHMENT_INDEX,
            Db::ALU_LIBS,
        ] {
            store.use_table(table.to_owned()).map_err(LaunchError::from)?;
        }

        Ok(Db { store })
    }

    pub fn retrieve<'a, H: 'a + sha256t::Tag, T: StrictDecode>(
        &mut self,
        table: &'static str,
        key: impl TaggedHash<'a, H> + 'a,
    ) -> Result<Option<T>, DaemonError> {
        let slice = key.into_inner();
        let slice = slice.into_inner();
        match self.store.retrieve(table.to_owned(), Slice32::from(slice))? {
            Some(data) => Ok(Some(T::strict_decode(data.as_ref())?)),
            None => Ok(None),
        }
    }

    pub fn retrieve_h<T: StrictDecode>(
        &mut self,
        table: &'static str,
        key: impl Hash<Inner = [u8; 32]>,
    ) -> Result<Option<T>, DaemonError> {
        let slice = *key.as_inner();
        match self.store.retrieve(table.to_owned(), Slice32::from(slice))? {
            Some(data) => Ok(Some(T::strict_decode(data.as_ref())?)),
            None => Ok(None),
        }
    }

    pub fn store<'a, H: 'a + sha256t::Tag>(
        &mut self,
        table: &'static str,
        key: impl TaggedHash<'a, H> + 'a,
        data: &impl StrictEncode,
    ) -> Result<(), DaemonError> {
        let slice = key.into_inner();
        let slice = slice.into_inner();
        self.store.store(table.to_owned(), Slice32::from(slice), data.strict_serialize()?)?;
        Ok(())
    }

    pub fn store_h(
        &mut self,
        table: &'static str,
        key: impl Hash<Inner = [u8; 32]>,
        data: &impl StrictEncode,
    ) -> Result<(), DaemonError> {
        let slice = *key.as_inner();
        self.store.store(table.to_owned(), Slice32::from(slice), data.strict_serialize()?)?;
        Ok(())
    }

    pub fn store_merge<'a, H: 'a + sha256t::Tag>(
        &mut self,
        table: &'static str,
        key: impl TaggedHash<'a, H> + Copy + 'a,
        new_obj: impl StrictEncode + StrictDecode + MergeReveal + Clone,
    ) -> Result<(), DaemonError> {
        let stored_obj = self.retrieve(table, key)?.unwrap_or_else(|| new_obj.clone());
        let obj = new_obj
            .merge_reveal(stored_obj)
            .expect("merge-revealed objects does not match; usually it means hacked database");
        self.store(Db::GENESIS, key, &obj)
    }

    pub fn store_merge_h(
        &mut self,
        table: &'static str,
        key: impl Hash<Inner = [u8; 32]>,
        new_obj: impl StrictEncode + StrictDecode + MergeReveal + Clone,
    ) -> Result<(), DaemonError> {
        let stored_obj = self.retrieve_h(table, key)?.unwrap_or_else(|| new_obj.clone());
        let obj = new_obj
            .merge_reveal(stored_obj)
            .expect("merge-revealed objects does not match; usually it means hacked database");
        self.store_h(Db::GENESIS, key, &obj)
    }
}