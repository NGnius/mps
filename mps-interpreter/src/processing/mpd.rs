use core::fmt::Debug;
use std::collections::VecDeque;
use std::net::{SocketAddr, TcpStream};
use std::iter::Iterator;

use mpd::Client;
use mpd::{Query, Term, Song};

use crate::lang::RuntimeMsg;
use crate::MpsItem;
use crate::lang::MpsTypePrimitive;

/// Music Player Daemon interface for interacting with it's database
pub trait MpsMpdQuerier: Debug {
    fn connect(&mut self, addr: SocketAddr) -> Result<(), RuntimeMsg>;

    fn search(&mut self, params: Vec<(&str, String)>) -> Result<VecDeque<MpsItem>, RuntimeMsg>;

    fn one_shot_search(&self, addr: SocketAddr, params: Vec<(&str, String)>) -> Result<VecDeque<MpsItem>, RuntimeMsg>;
}

#[derive(Default, Debug)]
pub struct MpsMpdExecutor {
    connection: Option<Client<TcpStream>>,
}

impl MpsMpdQuerier for MpsMpdExecutor {
    fn connect(&mut self, addr: SocketAddr) -> Result<(), RuntimeMsg> {
        self.connection = Some(Client::connect(addr).map_err(|e| RuntimeMsg(format!("MPD connection error: {}", e)))?);
        Ok(())
    }

    fn search(&mut self, params: Vec<(&str, String)>) -> Result<VecDeque<MpsItem>, RuntimeMsg> {
        if self.connection.is_none() {
            return Err(RuntimeMsg("MPD not connected".to_string()));
        }
        //let music_dir = self.connection.as_mut().unwrap().music_directory().map_err(|e| RuntimeMsg(format!("MPD command error: {}", e)))?;
        let mut query = Query::new();
        let mut query_mut = &mut query;
        for (term, value) in params {
            query_mut = query_mut.and(str_to_term(term), value);
        }
        let songs = self.connection.as_mut().unwrap().search(query_mut, None).map_err(|e| RuntimeMsg(format!("MPD search error: {}", e)))?;
        Ok(songs.into_iter().map(|x| song_to_item(x)).collect())
    }

    fn one_shot_search(&self, addr: SocketAddr, params: Vec<(&str, String)>) -> Result<VecDeque<MpsItem>, RuntimeMsg> {
        let mut connection = Client::connect(addr).map_err(|e| RuntimeMsg(format!("MPD connection error: {}", e)))?;
        //let music_dir = connection.music_directory().map_err(|e| RuntimeMsg(format!("MPD command error: {}", e)))?;
        let mut query = Query::new();
        let mut query_mut = &mut query;
        for (term, value) in params {
            query_mut = query_mut.and(str_to_term(term), value);
        }
        let songs = connection.search(query_mut, None).map_err(|e| RuntimeMsg(format!("MPD search error: {}", e)))?;
        Ok(songs.into_iter().map(|x| song_to_item(x)).collect())
    }
}

#[inline]
fn song_to_item(song: Song) -> MpsItem {
    let mut item = MpsItem::new();
    //item.set_field("filename", format!("{}{}{}", root_dir, std::path::MAIN_SEPARATOR, song.file).into());
    item.set_field("filename", format!("mpd://{}", song.file).into());
    if let Some(name) = song.name {
        item.set_field("name", name.into());
    }
    if let Some(title) = song.title {
        item.set_field("title", title.into());
    }

    /*
    if let Some(last_mod) = song.last_mod {
        item.set_field("last_modified", last_modified.into());
    }
    */

    if let Some(dur) = song.duration {
        item.set_field("duration", dur.num_seconds().into());
    }
    if let Some(place) = song.place {
        item.set_field("tracknumber", (place.pos as u64).into());
    }

    /*
    if let Some(range) = song.range {
        item.set_field("range", range.into());
    }
    */

    for (tag, value) in song.tags {
        item.set_field(&tag.to_lowercase(), MpsTypePrimitive::parse(value));
    }
    item
}

#[inline]
fn str_to_term<'a>(s: &'a str) -> Term<'a> {
    match s {
        "any" => Term::Any,
        "file" => Term::File,
        "base" => Term::Base,
        "lastmod" => Term::LastMod,
        x => Term::Tag(x.into())
    }
}
