use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use axum::body::Body;
use axum::extract::ws::{CloseFrame, Message};
use axum::extract::{DefaultBodyLimit, Multipart, Path, Query, WebSocketUpgrade};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{any, get, post};
use axum::Router;
use rand::random;
use serde::Serialize;
use tokio::sync::broadcast;
use zip::ZipArchive;

use crate::pack::{CharacterPack, CharacterPackCache};
use crate::utils::{SyncMutex, TimedResource};

mod pack;
mod utils;

const NUM_ROWS: usize = 4;
const NUM_COLS: usize = 6;
const NUM_CHARS: usize = NUM_ROWS * NUM_COLS;
const MAX_CHARACTER_PACK_CACHE_SIZE: usize = 1024 * 1024 * 1024;

#[derive(Clone, Copy, Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
enum GameEvent {
    Correct { user_id: u64, tries: usize },
    Incorrect { user_id: u64 },
}
impl GameEvent {
    fn user_id(&self) -> u64 {
        match self {
            Self::Correct { user_id, .. } => *user_id,
            Self::Incorrect { user_id } => *user_id,
        }
    }
}

struct GameState {
    pack: ZipArchive<Cursor<CharacterPack>>,
    characters: [usize; NUM_CHARS],
    events: broadcast::Sender<GameEvent>,
    p0: PlayerState,
    p1: PlayerState,
}
impl GameState {
    pub fn claim(&mut self, id: u64) -> bool {
        if self.p0.id == id {
            self.p0.claimed = true;
            true
        } else if self.p1.id == id {
            self.p1.claimed = true;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PlayerState {
    id: u64,
    claimed: bool,
    character: usize,
    incorrect_count: usize,
    correct: bool,
}
impl PlayerState {
    fn random() -> Self {
        Self {
            id: random(),
            claimed: false,
            character: rand::random_range(0..NUM_CHARS),
            incorrect_count: 0,
            correct: false,
        }
    }
}

#[derive(serde::Deserialize)]
struct GuessParams {
    row: usize,
    col: usize,
}

#[derive(Default)]
struct AppState {
    games: BTreeMap<u64, TimedResource<SyncMutex<GameState>>>,
    cache: CharacterPackCache,
}

#[tokio::main]

async fn main() {
    let games = Arc::new(SyncMutex::new(AppState::default()));

    let app =
        Router::new()
            .route(
                "/",
                get(|| async {
                    let mut res = StatusCode::OK.into_response();
                    *res.body_mut() = Body::from(include_str!("./index.html"));
                    res.headers_mut()
                        .insert("content-type", HeaderValue::from_static("text/html"));
                    res
                }),
            )
            .route(
                "/icon.png",
                get(|| async {
                    let mut res = StatusCode::OK.into_response();
                    *res.body_mut() = Body::from(&include_bytes!("../icon.png")[..]);
                    res.headers_mut()
                        .insert("content-type", HeaderValue::from_static("image/png"));
                    res
                }),
            )
            .route("/new_game", {
                let games = games.clone();
                post(|headers: HeaderMap, mut multipart: Multipart| async move {
                    async {
                        let game_id: u64 = random();
                        let mut pack = None;
                        if games.mutate(|g| g.cache.size()) >= MAX_CHARACTER_PACK_CACHE_SIZE {
                            let mut res = StatusCode::INSUFFICIENT_STORAGE.into_response();
                            *res.body_mut() = Body::from(include_str!("./overloaded.html"));
                            return Ok(res);
                        }
                        while let Some(field) = multipart.next_field().await? {
                            if field.name() == Some("character_pack") {
                                let bytes = field.bytes().await?;
                                pack = Some(ZipArchive::new(Cursor::new(
                                    games.mutate(|g| g.cache.cache(bytes)),
                                )));
                            }
                        }
                        let pack = pack.ok_or_else(|| anyhow!("character pack required"))??;
                        if pack.len() < NUM_CHARS {
                            return Err(anyhow!("not enough characters in pack"));
                        }
                        let uid = headers
                            .get("cookie")
                            .and_then(|c| c.to_str().ok())
                            .and_then(|c| {
                                c.split(";").find_map(|c| c.trim().strip_prefix("user_id="))
                            })
                            .and_then(|c| c.parse::<u64>().ok());
                        let mut p0 = PlayerState::random();
                        p0.id = uid.unwrap_or(p0.id);
                        let p0_id = p0.id;
                        games.mutate(|g| {
                            g.games.retain(|_, g| {
                                if let Some(game) = g.get() {
                                    !game.peek(|g| Some(g.p0.id) == uid || Some(g.p1.id) == uid)
                                // destroy the user's previous game
                                } else {
                                    false
                                }
                            });
                            g.games.insert(
                                game_id,
                                TimedResource::new(
                                    SyncMutex::new(GameState {
                                        characters: rand::seq::index::sample_array::<_, NUM_CHARS>(
                                            &mut rand::rng(),
                                            pack.len(),
                                        )
                                        .unwrap(),
                                        pack,
                                        events: broadcast::channel(10).0,
                                        p0,
                                        p1: PlayerState::random(),
                                    }),
                                    Duration::from_secs(60 * 60 * 4),
                                ),
                            );
                            eprintln!("{} games are active", g.games.len());
                        });
                        let mut res = Redirect::to(&format!("/game/{game_id}/")).into_response();
                        res.headers_mut().insert(
                            "set-cookie",
                            HeaderValue::from_str(&format!("user_id={p0_id}; SameSite=Strict"))?,
                        );
                        Ok(res)
                    }
                    .await
                    .map_err(|e: anyhow::Error| {
                        eprintln!("{e}");
                        eprintln!("{e:?}");
                        let mut res = StatusCode::INTERNAL_SERVER_ERROR.into_response();
                        *res.body_mut() = Body::from(include_str!("./oops.html"));
                        res.headers_mut()
                            .insert("content-type", HeaderValue::from_static("text/html"));
                        res
                    })
                })
            })
            .route(
                "/game/{game_id}",
                get(|Path::<u64>(game_id)| async move {
                    Redirect::permanent(&format!("/game/{game_id}/"))
                }),
            )
            .route("/game/{game_id}/", {
                let games = games.clone();
                get(|Path::<u64>(game_id), headers: HeaderMap| async move {
                    async {
                        let Some(game) =
                            games.peek(|g| g.games.get(&game_id).and_then(|g| g.get()))
                        else {
                            let mut res = StatusCode::NOT_FOUND.into_response();
                            *res.body_mut() = Body::from(include_str!("./not_found.html"));
                            res.headers_mut()
                                .insert("content-type", HeaderValue::from_static("text/html"));
                            return Ok(res);
                        };

                        let uid = headers
                            .get("cookie")
                            .and_then(|c| c.to_str().ok())
                            .and_then(|c| {
                                c.split(";").find_map(|c| c.trim().strip_prefix("user_id="))
                            })
                            .and_then(|c| c.parse::<u64>().ok())
                            .filter(|uid| game.mutate(|g| g.claim(*uid)));
                        if uid.is_some() {
                            let game_board = format!(
                                "<table>{}</table>",
                                (0..NUM_ROWS)
                                    .map(|row| format!(
                                        "<tr>{}</tr>",
                                        (0..NUM_COLS)
                                            .map(|col| format!(
                                                include_str!("./game-cell.html.template"),
                                                row = row,
                                                col = col,
                                            ))
                                            .collect::<String>()
                                    ))
                                    .collect::<String>()
                            );

                            let mut res = StatusCode::OK.into_response();
                            *res.body_mut() = Body::from(format!(
                                include_str!("./game.html.template"),
                                stylesheet = include_str!("./stylesheet.css"),
                                javascript = include_str!("./javascript.js"),
                                game_board = game_board
                            ));
                            res.headers_mut()
                                .insert("content-type", HeaderValue::from_static("text/html"));

                            Ok(res)
                        } else {
                            let Some(uid) = game.mutate(|g| {
                                if !g.p0.claimed {
                                    Some(g.p0.id)
                                } else if !g.p1.claimed {
                                    Some(g.p1.id)
                                } else {
                                    None
                                }
                            }) else {
                                let mut res = StatusCode::UNAUTHORIZED.into_response();
                                *res.body_mut() = Body::from(include_str!("./unauthorized.html"));
                                res.headers_mut()
                                    .insert("content-type", HeaderValue::from_static("text/html"));
                                return Ok(res);
                            };

                            let mut res = StatusCode::OK.into_response();
                            *res.body_mut() = Body::from(format!(
                                include_str!("./claim.html.template"),
                                user_id = uid
                            ));
                            res.headers_mut()
                                .insert("content-type", HeaderValue::from_static("text/html"));

                            Ok(res)
                        }
                    }
                    .await
                    .map_err(|e: anyhow::Error| {
                        eprintln!("{e}");
                        eprintln!("{e:?}");
                        let mut res = StatusCode::INTERNAL_SERVER_ERROR.into_response();
                        *res.body_mut() = Body::from(include_str!("./oops.html"));
                        res.headers_mut()
                            .insert("content-type", HeaderValue::from_static("text/html"));
                        res
                    })
                })
            })
            .route("/game/{game_id}/img-{image_id}", {
                let games = games.clone();
                get(
                    |Path::<(u64, String)>((game_id, image_id)), headers: HeaderMap| async move {
                        async {
                            let Some(game) =
                                games.peek(|g| g.games.get(&game_id).and_then(|g| g.get()))
                            else {
                                let mut res = StatusCode::NOT_FOUND.into_response();
                                *res.body_mut() = Body::from(include_str!("./not_found.html"));
                                return Ok(res);
                            };

                            let char_idx = if &*image_id == "mine" {
                                let Some(player_data) = headers
                                    .get("cookie")
                                    .and_then(|c| c.to_str().ok())
                                    .and_then(|c| {
                                        c.split(";").find_map(|c| c.trim().strip_prefix("user_id="))
                                    })
                                    .and_then(|c| c.parse::<u64>().ok())
                                    .and_then(|uid| {
                                        game.peek(|g| {
                                            if g.p0.id == uid {
                                                Some(g.p0)
                                            } else if g.p1.id == uid {
                                                Some(g.p1)
                                            } else {
                                                None
                                            }
                                        })
                                    })
                                else {
                                    let mut res = StatusCode::UNAUTHORIZED.into_response();
                                    *res.body_mut() =
                                        Body::from(include_str!("./unauthorized.html"));
                                    res.headers_mut().insert(
                                        "content-type",
                                        HeaderValue::from_static("text/html"),
                                    );
                                    return Ok(res);
                                };

                                player_data.character
                            } else {
                                let Some((row, col)) =
                                    image_id.split_once("_").and_then(|(row, col)| {
                                        Some((
                                            row.parse::<usize>().ok()?,
                                            col.parse::<usize>().ok()?,
                                        ))
                                    })
                                else {
                                    let mut res = StatusCode::NOT_FOUND.into_response();
                                    *res.body_mut() = Body::from(include_str!("./not_found.html"));
                                    res.headers_mut().insert(
                                        "content-type",
                                        HeaderValue::from_static("text/html"),
                                    );
                                    return Ok(res);
                                };

                                row * NUM_COLS + col
                            };

                            game.mutate(|g| {
                                let file_idx = g.characters[char_idx];
                                let mut file = g.pack.by_index(file_idx)?;
                                let mut res = StatusCode::OK.into_response();
                                let mut buf = Vec::new();
                                file.read_to_end(&mut buf)?;
                                if let Some(mime) = mime_guess::from_path(file.name()).first() {
                                    res.headers_mut().insert(
                                        "content-type",
                                        HeaderValue::from_str(&mime.to_string())?,
                                    );
                                }
                                *res.body_mut() = Body::from(buf);
                                Ok(res)
                            })
                        }
                        .await
                        .map_err(|e: anyhow::Error| {
                            eprintln!("{e}");
                            eprintln!("{e:?}");
                            let mut res = StatusCode::INTERNAL_SERVER_ERROR.into_response();
                            *res.body_mut() = Body::from(include_str!("./oops.html"));
                            res.headers_mut()
                                .insert("content-type", HeaderValue::from_static("text/html"));
                            res
                        })
                    },
                )
            })
            .route("/game/{game_id}/guess", {
                let games = games.clone();
                async fn guess(
                    games: Arc<SyncMutex<AppState>>,
                    game_id: u64,
                    GuessParams { row, col }: GuessParams,
                    headers: HeaderMap,
                ) -> Result<Response, anyhow::Error> {
                    let Some(game) = games.peek(|g| g.games.get(&game_id).and_then(|g| g.get()))
                    else {
                        let mut res = StatusCode::NOT_FOUND.into_response();
                        *res.body_mut() = Body::from(include_str!("./not_found.html"));
                        return Ok(res);
                    };

                    let Some((uid, other_player_data)) = headers
                        .get("cookie")
                        .and_then(|c| c.to_str().ok())
                        .and_then(|c| c.split(";").find_map(|c| c.trim().strip_prefix("user_id=")))
                        .and_then(|c| c.parse::<u64>().ok())
                        .and_then(|uid| {
                            game.peek(|g| {
                                if g.p0.id == uid {
                                    Some((uid, g.p1))
                                } else if g.p1.id == uid {
                                    Some((uid, g.p0))
                                } else {
                                    None
                                }
                            })
                        })
                    else {
                        let mut res = StatusCode::UNAUTHORIZED.into_response();
                        *res.body_mut() = Body::from(include_str!("./unauthorized.html"));
                        res.headers_mut()
                            .insert("content-type", HeaderValue::from_static("text/html"));
                        return Ok(res);
                    };

                    let correct = row * NUM_COLS + col == other_player_data.character;

                    game.mutate(|g| {
                        let player_data = if g.p0.id == uid { &mut g.p0 } else { &mut g.p1 };
                        if correct {
                            player_data.correct = true;
                        } else {
                            player_data.incorrect_count += 1;
                        }
                        let _ = g.events.send(if correct {
                            GameEvent::Correct {
                                user_id: uid,
                                tries: player_data.incorrect_count + 1,
                            }
                        } else {
                            GameEvent::Incorrect { user_id: uid }
                        });
                    });

                    let mut res = StatusCode::OK.into_response();
                    *res.body_mut() = Body::from(format!("{{ \"correct\": {correct} }} "));
                    res.headers_mut()
                        .insert("content-type", HeaderValue::from_static("application/json"));
                    Ok(res)
                }
                post(
                    |Path(game_id), Query(guess_params), headers: HeaderMap| async move {
                        guess(games, game_id, guess_params, headers).await.map_err(
                            |e: anyhow::Error| {
                                eprintln!("{e}");
                                eprintln!("{e:?}");
                                let mut res = StatusCode::INTERNAL_SERVER_ERROR.into_response();
                                *res.body_mut() = Body::from(include_str!("./oops.html"));
                                res.headers_mut()
                                    .insert("content-type", HeaderValue::from_static("text/html"));
                                res
                            },
                        )
                    },
                )
            })
            .route("/game/{game_id}/ws", {
                let games = games.clone();
                any(
                    |Path::<u64>(game_id), ws: WebSocketUpgrade, headers: HeaderMap| async move {
                        let Some(game) =
                            games.peek(|g| g.games.get(&game_id).and_then(|g| g.get()))
                        else {
                            let mut res = StatusCode::NOT_FOUND.into_response();
                            *res.body_mut() = Body::from(include_str!("./not_found.html"));
                            return res;
                        };
                        let Some(uid) = headers
                            .get("cookie")
                            .and_then(|c| c.to_str().ok())
                            .and_then(|c| {
                                c.split(";").find_map(|c| c.trim().strip_prefix("user_id="))
                            })
                            .and_then(|c| c.parse::<u64>().ok())
                            .filter(|uid| game.mutate(|g| g.claim(*uid)))
                        else {
                            let mut res = StatusCode::UNAUTHORIZED.into_response();
                            *res.body_mut() = Body::from(include_str!("./unauthorized.html"));
                            res.headers_mut()
                                .insert("content-type", HeaderValue::from_static("text/html"));
                            return res;
                        };
                        let mut sub = game.peek(|g| g.events.subscribe());
                        ws.on_upgrade(move |mut ws| async move {
                            if let Err(e) = async {
                                loop {
                                    match sub.recv().await {
                                        Ok(e) if e.user_id() != uid => {
                                            ws.send(Message::Text(
                                                serde_json::to_string(&e)?.into(),
                                            ))
                                            .await?;
                                        }
                                        Err(broadcast::error::RecvError::Closed) => {
                                            break;
                                        }
                                        _ => (),
                                    }
                                }
                                ws.send(Message::Close(Some(CloseFrame {
                                    code: 1000,
                                    reason: "complete".into(),
                                })))
                                .await?;
                                ws.recv().await;
                                drop(ws);

                                Ok::<_, anyhow::Error>(())
                            }
                            .await
                            {
                                eprintln!("{e}");
                                eprintln!("{e:?}");
                            }
                        })
                    },
                )
            })
            .layer(DefaultBodyLimit::max(1024 * 1024 * 128));
    // on load - select who they are guessing
    // show other player's link

    // run our app with hyper, listening globally on port 3000

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
