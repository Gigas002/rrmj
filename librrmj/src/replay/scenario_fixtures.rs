//! Hand-authored scenario builders for `examples/scenarios/*.json`.
//!
//! Run `cargo test -p librrmj --features serde write_all_scenario_fixtures -- --ignored --nocapture`
//! to refresh committed fixtures.

#![cfg(all(test, feature = "serde"))]

use std::fs;
use std::path::PathBuf;

use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::action::{Action, KanIntent};
use crate::ai::MatchSetup;
use crate::event::Event;
use crate::game::{AbortiveDrawKind, Game, MatchLength, MatchPhase, RoundWind};
use crate::hand::{Concealed, Hand, Meld};
use crate::replay::{FORMAT_VERSION, MatchRecording, MatchStatus, PlayerSetup, RecordingMeta};
use crate::rules::RulesConfig;
use crate::scoring::Yaku;
use crate::state::{HandEndReason, HandPhase, HandState};
use crate::test_util::fixtures::tenpai_after_draw_p2;
use crate::tile::{Dragon, Suit, Tile, Wind};
use crate::wall::Wall;

struct ScenarioSpec {
    filename: &'static str,
    human_seat: usize,
    meta: RecordingMeta,
    expected: Option<Vec<Action>>,
    expected_yaku: Option<Vec<Yaku>>,
    build: fn() -> MatchRecording,
}

pub fn write_all() {
    let dir = scenarios_dir();
    fs::create_dir_all(&dir).expect("create scenarios dir");

    for spec in all_specs() {
        let mut recording = (spec.build)();
        recording.meta = spec.meta;
        recording.human_seat = Some(spec.human_seat);
        recording.expected_legal_actions = spec.expected.clone();
        recording.expected_yaku = spec.expected_yaku.clone();
        let setup = MatchSetup::all_medium(recording.seed).with_human_seat(spec.human_seat);
        recording.players = std::array::from_fn(|seat| PlayerSetup::from_match_setup(&setup, seat));
        recording
            .validate()
            .unwrap_or_else(|err| panic!("validate {}: {err}", spec.filename));
        let path = dir.join(spec.filename);
        fs::write(path, recording.to_json().unwrap()).expect("write scenario");
    }
}

fn scenarios_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("examples")
        .join("scenarios")
}

fn all_specs() -> Vec<ScenarioSpec> {
    vec![
        spec(
            "dealer_first_turn.json",
            0,
            meta(
                "Dealer first turn",
                "East opens the hand with 14 tiles in discard phase.",
                &["turn", "discard"],
            ),
            None,
            None,
            dealer_first_turn,
        ),
        spec(
            "draw_after_discard.json",
            1,
            meta(
                "Draw after discard",
                "South's turn: draw phase after East discarded and all passed.",
                &["turn", "draw"],
            ),
            Some(vec![Action::Draw]),
            None,
            draw_after_discard,
        ),
        spec(
            "normal_discard.json",
            0,
            meta(
                "Normal discard",
                "Dealer must choose a tile to discard.",
                &["discard"],
            ),
            None,
            None,
            normal_discard,
        ),
        spec(
            "pon_reaction.json",
            1,
            meta(
                "Pon call available",
                "South can pon 2m from East's discard.",
                &["calls", "pon", "reaction"],
            ),
            Some(vec![Action::Pon, Action::Pass]),
            None,
            pon_reaction,
        ),
        spec(
            "chi_kamicha.json",
            1,
            meta(
                "Chi from kamicha",
                "South (kamicha) can chi 1m-2m-3m on East's 2m discard.",
                &["calls", "chi", "reaction"],
            ),
            None,
            None,
            chi_kamicha,
        ),
        spec(
            "chi_shimocha_illegal.json",
            2,
            meta(
                "Chi blocked for shimocha",
                "West cannot chi — only kamicha may call chi.",
                &["calls", "chi", "reaction"],
            ),
            Some(vec![Action::Pass]),
            None,
            chi_shimocha_blocked,
        ),
        spec(
            "open_kan.json",
            1,
            meta(
                "Open kan (daiminkan)",
                "South can declare open kan on 6s; reveals dora and rinshan draw.",
                &["calls", "kan", "dora", "reaction"],
            ),
            Some(vec![Action::Kan(KanIntent::Open), Action::Pass]),
            None,
            open_kan,
        ),
        spec(
            "closed_kan.json",
            0,
            meta(
                "Closed kan (ankan)",
                "East can declare closed kan on four 3p.",
                &["calls", "kan", "dora"],
            ),
            None,
            None,
            closed_kan,
        ),
        spec(
            "rinshan_discard.json",
            1,
            meta(
                "Rinshan discard",
                "After open kan, caller holds 15 tiles and must discard.",
                &["calls", "kan", "rinshan", "discard"],
            ),
            None,
            None,
            rinshan_discard,
        ),
        spec(
            "ron_on_discard.json",
            1,
            meta(
                "Ron on discard",
                "South is tenpai and can ron East's 2p discard.",
                &["ron", "scoring", "reaction"],
            ),
            Some(vec![Action::Ron, Action::Pass]),
            Some(vec![Yaku::Tanyao]),
            ron_on_discard,
        ),
        spec(
            "ron_over_pon.json",
            2,
            meta(
                "Pon vs chi priority",
                "West can pon while South can chi — pon wins over chi.",
                &["calls", "reaction"],
            ),
            None,
            None,
            ron_over_pon,
        ),
        spec(
            "non_dealer_tsumo.json",
            1,
            meta(
                "Non-dealer tsumo",
                "South wins on tsumo (tanyao).",
                &["tsumo", "scoring"],
            ),
            Some(vec![Action::Tsumo]),
            Some(vec![Yaku::MenzenTsumo, Yaku::Tanyao]),
            non_dealer_tsumo,
        ),
        spec(
            "dealer_tsumo.json",
            0,
            meta(
                "Dealer tsumo (tanyao)",
                "East wins on tsumo; hand ends and next hand is dealt.",
                &["tsumo", "scoring", "hand-end"],
            ),
            Some(vec![Action::Tsumo]),
            Some(vec![Yaku::MenzenTsumo, Yaku::Tanyao]),
            dealer_tsumo_after_honba,
        ),
        spec(
            "riichi_declare.json",
            2,
            meta(
                "Riichi declaration",
                "West is tenpai and can declare riichi on 2p.",
                &["riichi", "discard"],
            ),
            None,
            None,
            riichi_declare,
        ),
        spec(
            "reaction_pass.json",
            2,
            meta(
                "Reaction pass",
                "Multiple seats may pass on a safe discard.",
                &["reaction"],
            ),
            Some(vec![Action::Pass]),
            None,
            reaction_pass,
        ),
        spec(
            "exhaustive_draw.json",
            0,
            meta(
                "Exhaustive draw",
                "Wall empty; hand ends in exhaustive draw.",
                &["draw", "hand-end"],
            ),
            None,
            None,
            exhaustive_draw,
        ),
        spec(
            "honba_carry.json",
            0,
            meta(
                "Honba carry",
                "Dealer won last hand; honba stick on table at 1.",
                &["match-flow", "scoring"],
            ),
            None,
            None,
            honba_carry,
        ),
        spec(
            "south_round.json",
            0,
            meta(
                "South round",
                "Match in South 1 after East round completed.",
                &["match-flow"],
            ),
            None,
            None,
            south_round,
        ),
        spec(
            "nine_terminals.json",
            0,
            meta(
                "Nine terminals abortive",
                "Dealer may declare nine-terminals abortive draw.",
                &["abortive", "draw"],
            ),
            Some(vec![Action::AbortiveNineTerminals]),
            None,
            nine_terminals,
        ),
        spec(
            "multi_reaction.json",
            1,
            meta(
                "Multiple reactions",
                "Pon and chi both legal; pon from seat 2 should win priority.",
                &["reaction", "calls"],
            ),
            None,
            None,
            multi_reaction,
        ),
        spec(
            "pinfu_tsumo.json",
            0,
            meta(
                "Pinfu menzen tsumo",
                "East wins on tsumo with a menzen pinfu hand (terminals, no tanyao).",
                &["tsumo", "scoring", "pinfu"],
            ),
            Some(vec![Action::Tsumo]),
            Some(vec![Yaku::Pinfu, Yaku::MenzenTsumo]),
            pinfu_tsumo,
        ),
        spec(
            "yakuhai_ron.json",
            1,
            meta(
                "Yakuhai ron",
                "South rons East's east-wind discard for yakuhai.",
                &["ron", "scoring", "yakuhai"],
            ),
            None,
            None,
            yakuhai_ron,
        ),
        spec(
            "menzen_tsumo.json",
            2,
            meta(
                "Menzen tsumo",
                "West wins on tsumo with closed hand (tanyao + menzen tsumo).",
                &["tsumo", "scoring", "menzen"],
            ),
            Some(vec![Action::Tsumo]),
            Some(vec![Yaku::MenzenTsumo, Yaku::Tanyao]),
            menzen_tsumo,
        ),
        spec(
            "chi_left.json",
            1,
            meta(
                "Chi left (shuntsu)",
                "South can chi 4p-5p-6p on East's 6p discard.",
                &["calls", "chi", "reaction"],
            ),
            None,
            None,
            chi_left,
        ),
        spec(
            "riichi_tsumo.json",
            2,
            meta(
                "Riichi tsumo",
                "West wins riichi + menzen tsumo + tanyao on self-draw.",
                &["riichi", "tsumo", "scoring"],
            ),
            Some(vec![Action::Tsumo]),
            Some(vec![Yaku::Riichi, Yaku::MenzenTsumo, Yaku::Tanyao]),
            riichi_tsumo,
        ),
        spec(
            "tanyao_ron.json",
            1,
            meta(
                "Tanyao ron",
                "South rons 2p for tanyao.",
                &["ron", "scoring", "tanyao"],
            ),
            Some(vec![Action::Ron, Action::Pass]),
            Some(vec![Yaku::Tanyao]),
            tanyao_ron,
        ),
        spec(
            "chiitoitsu_tsumo.json",
            0,
            meta(
                "Chiitoitsu tsumo",
                "East wins chiitoitsu + menzen tsumo + tanyao.",
                &["tsumo", "scoring", "chiitoitsu"],
            ),
            Some(vec![Action::Tsumo]),
            Some(vec![Yaku::Chiitoitsu, Yaku::MenzenTsumo, Yaku::Tanyao]),
            chiitoitsu_tsumo,
        ),
        spec(
            "yakuhai_red_ron.json",
            3,
            meta(
                "Red dragon yakuhai ron",
                "North rons red dragon for yakuhai.",
                &["ron", "scoring", "yakuhai"],
            ),
            Some(vec![Action::Ron, Action::Pass]),
            Some(vec![Yaku::Yakuhai]),
            yakuhai_red_ron,
        ),
        spec(
            "open_tanyao_ron.json",
            1,
            meta(
                "Open tanyao ron",
                "South rons with open pon — tanyao only.",
                &["ron", "scoring", "tanyao", "calls"],
            ),
            Some(vec![Action::Ron, Action::Pass]),
            Some(vec![Yaku::Tanyao]),
            open_tanyao_ron,
        ),
        spec(
            "pinfu_ron.json",
            0,
            meta(
                "Pinfu ron",
                "East rons for menzen pinfu.",
                &["ron", "scoring", "pinfu"],
            ),
            Some(vec![Action::Ron, Action::Pass]),
            Some(vec![Yaku::Pinfu]),
            pinfu_ron,
        ),
        spec(
            "chi_middle.json",
            1,
            meta(
                "Chi middle (shuntsu)",
                "South can chi 4p-5p-6p on East's 5p discard.",
                &["calls", "chi", "reaction"],
            ),
            Some(vec![
                Action::Chi {
                    tiles: [Tile::pin(4), Tile::pin(5), Tile::pin(6)],
                },
                Action::Pass,
            ]),
            None,
            chi_middle,
        ),
        spec(
            "chi_right.json",
            1,
            meta(
                "Chi right (shuntsu)",
                "South can chi 3p-4p-5p on East's 3p discard.",
                &["calls", "chi", "reaction"],
            ),
            Some(vec![
                Action::Chi {
                    tiles: [Tile::pin(3), Tile::pin(4), Tile::pin(5)],
                },
                Action::Pass,
            ]),
            None,
            chi_right,
        ),
        spec(
            "kakan.json",
            0,
            meta(
                "Kakan (pon upgrade)",
                "East can upgrade an open pon to kan with the fourth matching tile.",
                &["calls", "kan", "dora"],
            ),
            Some(vec![Action::Kan(KanIntent::Added { meld_index: 0 })]),
            None,
            kakan,
        ),
        spec(
            "chankan_ron.json",
            2,
            meta(
                "Chankan ron",
                "West rons the tile added during an opponent's kakan.",
                &["calls", "kan", "ron", "scoring", "reaction"],
            ),
            Some(vec![Action::Ron, Action::Pass]),
            Some(vec![Yaku::Chankan]),
            chankan_ron,
        ),
        spec(
            "double_ron.json",
            1,
            meta(
                "Double ron",
                "South and North can both ron the same 2p discard.",
                &["ron", "scoring", "reaction"],
            ),
            Some(vec![Action::Ron, Action::Pass]),
            Some(vec![Yaku::Tanyao]),
            double_ron,
        ),
        spec(
            "triple_ron.json",
            1,
            meta(
                "Triple ron",
                "Three seats can ron the same discard when triple ron is enabled.",
                &["ron", "scoring", "reaction"],
            ),
            Some(vec![Action::Ron, Action::Pass]),
            Some(vec![Yaku::Tanyao]),
            triple_ron,
        ),
        spec(
            "furiten_temporary.json",
            1,
            meta(
                "Temporary furiten",
                "South passed on a winning discard; temporary furiten blocks ron on that tile.",
                &["furiten", "ron", "reaction"],
            ),
            Some(vec![Action::Pass]),
            None,
            furiten_temporary,
        ),
        spec(
            "furiten_cleared.json",
            1,
            meta(
                "Temporary furiten cleared",
                "After drawing, temporary furiten clears and South can ron again.",
                &["furiten", "ron", "reaction", "draw"],
            ),
            Some(vec![Action::Ron, Action::Pass]),
            Some(vec![Yaku::Tanyao]),
            furiten_cleared,
        ),
        spec(
            "furiten_riichi.json",
            1,
            meta(
                "Riichi furiten",
                "Riichi furiten persists after a draw; South cannot ron a passed tile.",
                &["furiten", "riichi", "ron", "reaction"],
            ),
            Some(vec![Action::Pass]),
            None,
            furiten_riichi,
        ),
        spec(
            "dora_kan_chain.json",
            1,
            meta(
                "Dora after kan chain",
                "Two kans revealed three dora indicators on the dead wall.",
                &["dora", "kan", "calls"],
            ),
            None,
            None,
            dora_kan_chain,
        ),
        spec(
            "ura_dora_riichi.json",
            2,
            meta(
                "Ura dora (riichi)",
                "Closed riichi win with ura-dora indicators exposed on the dead wall.",
                &["dora", "riichi", "tsumo", "scoring"],
            ),
            Some(vec![Action::Tsumo]),
            Some(vec![Yaku::Riichi, Yaku::MenzenTsumo, Yaku::Tanyao]),
            ura_dora_riichi,
        ),
        spec(
            "aka_dora_on.json",
            0,
            meta(
                "Aka dora enabled",
                "Dealer holds red fives; rules config has aka dora enabled.",
                &["dora", "scoring"],
            ),
            None,
            None,
            aka_dora_on,
        ),
        spec(
            "aka_dora_off.json",
            0,
            meta(
                "Aka dora disabled",
                "Same red-five tiles with aka dora turned off in rules config.",
                &["dora", "scoring"],
            ),
            None,
            None,
            aka_dora_off,
        ),
        spec(
            "mangan_ron.json",
            0,
            meta(
                "Mangan ron",
                "East rons for five han (mangan threshold).",
                &["ron", "scoring", "hand-end"],
            ),
            Some(vec![Action::Ron, Action::Pass]),
            Some(vec![Yaku::Chiitoitsu, Yaku::DoubleRiichi, Yaku::Tanyao]),
            mangan_ron,
        ),
        spec(
            "honba_scoring.json",
            1,
            meta(
                "Honba on table at win",
                "South rons with two honba sticks on the table.",
                &["scoring", "match-flow", "ron"],
            ),
            Some(vec![Action::Ron, Action::Pass]),
            Some(vec![Yaku::Tanyao]),
            honba_scoring,
        ),
        spec(
            "exhaustive_draw_mixed.json",
            0,
            meta(
                "Exhaustive draw (mixed tenpai)",
                "Two seats tenpai and two noten when the live wall is exhausted.",
                &["draw", "hand-end", "scoring"],
            ),
            None,
            None,
            exhaustive_draw_mixed,
        ),
        spec(
            "four_winds_abortive.json",
            3,
            meta(
                "Four winds abortive",
                "All four first discards are the same wind; abortive draw on the fourth.",
                &["abortive", "draw"],
            ),
            None,
            None,
            four_winds_abortive,
        ),
        spec(
            "four_kongs_abortive.json",
            0,
            meta(
                "Four kongs abortive",
                "Fourth kan declaration ends the hand in an abortive draw.",
                &["abortive", "draw", "kan"],
            ),
            None,
            None,
            four_kongs_abortive,
        ),
        spec(
            "four_riichis_abortive.json",
            3,
            meta(
                "Four riichis abortive",
                "Fourth riichi declaration ends the hand in an abortive draw.",
                &["abortive", "draw", "riichi"],
            ),
            None,
            None,
            four_riichis_abortive,
        ),
        spec(
            "match_finished.json",
            0,
            meta(
                "Match finished",
                "East-only match completed; recording has match_status finished.",
                &["match-flow"],
            ),
            None,
            None,
            match_finished,
        ),
    ]
}

fn spec(
    filename: &'static str,
    human_seat: usize,
    meta: RecordingMeta,
    expected: Option<Vec<Action>>,
    expected_yaku: Option<Vec<Yaku>>,
    build: fn() -> MatchRecording,
) -> ScenarioSpec {
    ScenarioSpec {
        filename,
        human_seat,
        meta,
        expected,
        expected_yaku,
        build,
    }
}

fn meta(title: &str, description: &str, tags: &[&str]) -> RecordingMeta {
    RecordingMeta {
        title: Some(title.into()),
        description: Some(description.into()),
        tags: tags.iter().map(|t| (*t).into()).collect(),
        ..RecordingMeta::default()
    }
}

fn capture_match(
    game: &Game,
    human_seat: usize,
    meta: RecordingMeta,
    expected: Option<Vec<Action>>,
) -> MatchRecording {
    let setup = MatchSetup::all_medium(game.seed()).with_human_seat(human_seat);
    let mut recording = MatchRecording::capture(game, &setup, human_seat, 300, 30_000, 5_000, meta);
    recording.expected_legal_actions = expected;
    recording
}

#[allow(clippy::too_many_arguments)]
fn recording_from_hand(
    hand: HandState,
    seed: u64,
    human_seat: usize,
    meta: RecordingMeta,
    expected: Option<Vec<Action>>,
    hand_index: u32,
    round_wind: RoundWind,
    kyoku: u8,
) -> MatchRecording {
    let setup = MatchSetup::all_medium(seed).with_human_seat(human_seat);
    let players = std::array::from_fn(|seat| PlayerSetup::from_match_setup(&setup, seat));
    MatchRecording {
        format_version: FORMAT_VERSION,
        meta,
        rules_profile: hand.config().profile,
        rules_config: hand.config().clone(),
        seed,
        players,
        human_seat: Some(human_seat),
        cpu_step_delay_ms: Some(300),
        turn_timer_ms: Some(30_000),
        response_timer_ms: Some(5_000),
        match_status: MatchStatus::InProgress,
        dealer: hand.dealer(),
        round_wind,
        kyoku,
        honba: hand.honba(),
        scores: *hand.scores(),
        table_riichi_sticks: hand.table_riichi_sticks(),
        hand_index,
        match_phase: MatchPhase::InHand,
        hand: hand.to_snapshot(),
        events: vec![
            Event::Dealt {
                dealer: hand.dealer(),
            },
            Event::HandStarted {
                dealer: hand.dealer(),
                round_wind,
                kyoku,
                honba: hand.honba(),
            },
        ],
        event_index: 1,
        expected_legal_actions: expected,
        expected_yaku: None,
    }
}

// --- tile helpers (mirrors state/tests.rs) ---

fn winning_tanyao_tiles() -> Vec<Tile> {
    vec![
        Tile::man(2),
        Tile::man(3),
        Tile::man(4),
        Tile::pin(3),
        Tile::pin(4),
        Tile::pin(5),
        Tile::sou(6),
        Tile::sou(7),
        Tile::sou(8),
        Tile::sou(5),
        Tile::sou(5),
        Tile::sou(5),
        Tile::pin(2),
        Tile::pin(2),
    ]
}

fn tenpai_waiting_on_p2() -> Vec<Tile> {
    let mut hand = winning_tanyao_tiles();
    hand.pop();
    hand
}

fn build_hand(seat: usize, mut tiles: Vec<Tile>) -> Vec<Tile> {
    let mut n = 0usize;
    while tiles.len() < 13 {
        let rank = ((n + seat) % 9) + 1;
        let suit = match ((n + seat) / 9) % 3 {
            0 => Suit::Man,
            1 => Suit::Pin,
            _ => Suit::Sou,
        };
        let candidate = Tile::numbered(suit, rank as u8);
        if !tiles.contains(&candidate) {
            tiles.push(candidate);
        }
        n += 1;
    }
    tiles
}

fn start_reaction(seed: u64, dealer: usize, discarded: Tile, hands: [Vec<Tile>; 4]) -> HandState {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(seed));
    let deal = wall.deal(dealer).unwrap();
    let mut state = HandState::from_deal(wall, deal, rules);

    let mut configured = hands;
    if !configured[dealer].contains(&discarded) {
        configured[dealer].push(discarded);
    }
    for (seat, tiles) in configured.into_iter().enumerate() {
        state.set_concealed(seat, build_hand(seat, tiles));
    }
    state.apply(dealer, Action::Discard(discarded)).unwrap();
    state
}

fn pass_all_except(state: &mut HandState, skip: usize) {
    for seat in 0..4 {
        if seat != skip
            && state.phase() == HandPhase::Reaction
            && state.legal_actions_for(seat).contains(&Action::Pass)
        {
            state.apply(seat, Action::Pass).unwrap();
        }
    }
}

fn force_tsumo_win(game: &mut Game, seat: usize) {
    let hand = game.hand_mut();
    hand.set_concealed(seat, winning_tanyao_tiles());
    hand.last_draw = Some(Tile::pin(2));
    hand.is_dealer_first_turn = false;
}

fn force_ron_win(game: &mut Game, winner: usize) {
    crate::test_util::fixtures::force_ron_win(game, winner);
}

// --- scenario builders ---

fn dealer_first_turn() -> MatchRecording {
    let seed = 1001;
    let game = Game::new(RulesConfig::standard(), seed).unwrap();
    capture_match(&game, 0, meta("", "", &[]), None)
}

fn draw_after_discard() -> MatchRecording {
    let seed = 1002;
    let mut game = Game::new(RulesConfig::standard(), seed).unwrap();
    let tile = game.hand().hand(0).concealed().tiles()[0];
    game.apply_action(0, Action::Discard(tile)).unwrap();
    for seat in 1..4 {
        game.apply_action(seat, Action::Pass).unwrap();
    }
    capture_match(&game, 1, meta("", "", &[]), Some(vec![Action::Draw]))
}

fn normal_discard() -> MatchRecording {
    dealer_first_turn()
}

fn pon_reaction() -> MatchRecording {
    let discarded = Tile::man(2);
    let hand = start_reaction(
        2010,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::man(2), Tile::man(2), Tile::pin(9)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(6)],
        ],
    );
    recording_from_hand(
        hand,
        2010,
        1,
        meta("", "", &[]),
        Some(vec![Action::Pon, Action::Pass]),
        0,
        RoundWind::East,
        1,
    )
}

fn chi_kamicha() -> MatchRecording {
    let discarded = Tile::man(2);
    let hand = start_reaction(
        2011,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::man(1), Tile::man(3), Tile::pin(9)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(6)],
        ],
    );
    recording_from_hand(
        hand,
        2011,
        1,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn chi_shimocha_blocked() -> MatchRecording {
    let discarded = Tile::man(2);
    let hand = start_reaction(
        2012,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::man(1), Tile::man(3), Tile::pin(9)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(6)],
        ],
    );
    recording_from_hand(
        hand,
        2012,
        2,
        meta("", "", &[]),
        Some(vec![Action::Pass]),
        0,
        RoundWind::East,
        1,
    )
}

fn open_kan() -> MatchRecording {
    let discarded = Tile::sou(6);
    let hand = start_reaction(
        2013,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::sou(6), Tile::sou(6), Tile::sou(6), Tile::pin(9)],
            vec![Tile::man(1), Tile::man(2), Tile::man(3)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(7)],
        ],
    );
    recording_from_hand(
        hand,
        2013,
        1,
        meta("", "", &[]),
        Some(vec![Action::Kan(KanIntent::Open), Action::Pass]),
        0,
        RoundWind::East,
        1,
    )
}

fn closed_kan() -> MatchRecording {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2014));
    let deal = wall.deal(0).unwrap();
    let mut hand = HandState::from_deal(wall, deal, rules);
    hand.set_concealed(
        0,
        vec![
            Tile::pin(3),
            Tile::pin(3),
            Tile::pin(3),
            Tile::pin(3),
            Tile::man(1),
            Tile::man(2),
            Tile::man(4),
            Tile::man(5),
            Tile::man(6),
            Tile::man(7),
            Tile::man(8),
            Tile::man(9),
            Tile::sou(1),
            Tile::sou(2),
        ],
    );
    recording_from_hand(
        hand,
        2014,
        0,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn rinshan_discard() -> MatchRecording {
    let discarded = Tile::sou(6);
    let mut hand = start_reaction(
        2015,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::sou(6), Tile::sou(6), Tile::sou(6), Tile::pin(9)],
            vec![Tile::man(1), Tile::man(2), Tile::man(3)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(7)],
        ],
    );
    hand.apply(1, Action::Kan(KanIntent::Open)).unwrap();
    pass_all_except(&mut hand, 1);
    recording_from_hand(
        hand,
        2015,
        1,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn ron_on_discard() -> MatchRecording {
    let discarded = Tile::pin(2);
    let hand = start_reaction(
        2020,
        0,
        discarded,
        [
            vec![Tile::man(1), Tile::man(3), Tile::pin(1)],
            vec![Tile::pin(3), Tile::pin(4), Tile::sou(6)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(6)],
        ],
    );
    let mut hand = hand;
    hand.set_concealed(1, tenpai_waiting_on_p2());
    recording_from_hand(
        hand,
        2020,
        1,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn ron_over_pon() -> MatchRecording {
    let discarded = Tile::man(2);
    let mut hand = start_reaction(
        2021,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::man(1), Tile::man(3), Tile::pin(9)],
            vec![Tile::man(2), Tile::man(2), Tile::pin(8)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(6)],
        ],
    );
    hand.set_concealed(2, tenpai_waiting_on_p2());
    recording_from_hand(
        hand,
        2021,
        2,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn non_dealer_tsumo() -> MatchRecording {
    let seed = 2030;
    let mut game = Game::new(RulesConfig::standard(), seed).unwrap();
    let tile = game.hand().hand(0).concealed().tiles()[0];
    game.apply_action(0, Action::Discard(tile)).unwrap();
    for seat in 1..4 {
        game.apply_action(seat, Action::Pass).unwrap();
    }
    game.apply_action(1, Action::Draw).unwrap();
    let hand = game.hand_mut();
    hand.set_concealed(1, winning_tanyao_tiles());
    hand.last_draw = Some(Tile::pin(2));
    capture_match(&game, 1, meta("", "", &[]), None)
}

fn dealer_tsumo_after_honba() -> MatchRecording {
    let seed = 42;
    let mut game = Game::new(RulesConfig::standard(), seed).unwrap();
    let dealer = game.dealer();
    force_tsumo_win(&mut game, dealer);
    game.apply_action(dealer, Action::Tsumo).unwrap();
    let dealer = game.dealer();
    force_tsumo_win(&mut game, dealer);
    capture_match(&game, 0, meta("", "", &[]), None)
}

fn riichi_declare() -> MatchRecording {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2040));
    let deal = wall.deal(2).unwrap();
    let mut hand = HandState::from_deal(wall, deal, rules);
    hand.set_concealed(2, winning_tanyao_tiles());
    hand.last_draw = Some(Tile::pin(2));
    recording_from_hand(
        hand,
        2040,
        2,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn reaction_pass() -> MatchRecording {
    let discarded = Tile::man(5);
    let hand = start_reaction(
        2050,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::man(1), Tile::man(3), Tile::pin(9)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(6)],
        ],
    );
    recording_from_hand(
        hand,
        2050,
        2,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn exhaustive_draw() -> MatchRecording {
    let seed = 2060;
    let mut game = Game::new(RulesConfig::standard(), seed).unwrap();
    game.hand_mut()
        .play_out_discards(|state, seat| state.hand(seat).concealed().tiles()[0])
        .unwrap();
    capture_match(&game, 0, meta("", "", &[]), None)
}

fn honba_carry() -> MatchRecording {
    let seed = 2070;
    let mut game = Game::new(RulesConfig::standard(), seed).unwrap();
    let dealer = game.dealer();
    force_tsumo_win(&mut game, dealer);
    game.apply_action(dealer, Action::Tsumo).unwrap();
    capture_match(&game, 0, meta("", "", &[]), None)
}

fn south_round() -> MatchRecording {
    let mut config = RulesConfig::standard();
    config.match_length = MatchLength::Hanchan;
    let mut game = Game::new(config, 2080).unwrap();

    for _ in 0..4 {
        let winner = (game.dealer() + 1) % 4;
        force_ron_win(&mut game, winner);
    }
    capture_match(&game, 0, meta("", "", &[]), None)
}

fn nine_terminals() -> MatchRecording {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2100));
    let deal = wall.deal(0).unwrap();
    let mut hand = HandState::from_deal(wall, deal, rules);
    hand.set_concealed(
        0,
        vec![
            Tile::man(1),
            Tile::man(1),
            Tile::man(9),
            Tile::man(9),
            Tile::pin(1),
            Tile::pin(9),
            Tile::sou(1),
            Tile::sou(9),
            Tile::wind(Wind::East),
            Tile::wind(Wind::East),
            Tile::wind(Wind::South),
            Tile::wind(Wind::South),
            Tile::wind(Wind::West),
            Tile::wind(Wind::West),
        ],
    );
    recording_from_hand(
        hand,
        2100,
        0,
        meta("", "", &[]),
        Some(vec![Action::AbortiveNineTerminals]),
        0,
        RoundWind::East,
        1,
    )
}

fn multi_reaction() -> MatchRecording {
    let discarded = Tile::man(2);
    let hand = start_reaction(
        2110,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::man(1), Tile::man(3), Tile::pin(9)],
            vec![Tile::man(2), Tile::man(2), Tile::pin(8)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(6)],
        ],
    );
    recording_from_hand(
        hand,
        2110,
        1,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn pinfu_tiles() -> Vec<Tile> {
    vec![
        Tile::man(1),
        Tile::man(2),
        Tile::man(3),
        Tile::man(5),
        Tile::man(6),
        Tile::man(7),
        Tile::pin(3),
        Tile::pin(4),
        Tile::pin(5),
        Tile::pin(8),
        Tile::pin(8),
        Tile::sou(2),
        Tile::sou(3),
        Tile::sou(4),
    ]
}

fn yakuhai_tenpai() -> Vec<Tile> {
    vec![
        Tile::man(2),
        Tile::man(3),
        Tile::man(4),
        Tile::pin(5),
        Tile::pin(6),
        Tile::pin(7),
        Tile::sou(6),
        Tile::sou(7),
        Tile::sou(8),
        Tile::wind(Wind::East),
        Tile::wind(Wind::East),
        Tile::pin(2),
        Tile::pin(3),
    ]
}

fn pinfu_tsumo() -> MatchRecording {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2121));
    let deal = wall.deal(0).unwrap();
    let mut hand = HandState::from_deal(wall, deal, rules);
    hand.set_concealed(0, pinfu_tiles());
    hand.last_draw = Some(Tile::sou(4));
    hand.is_dealer_first_turn = false;
    recording_from_hand(
        hand,
        2121,
        0,
        meta("", "", &[]),
        Some(vec![Action::Tsumo]),
        0,
        RoundWind::East,
        1,
    )
}

fn yakuhai_ron() -> MatchRecording {
    let discarded = Tile::wind(Wind::East);
    let mut hand = start_reaction(
        2122,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::man(2), Tile::man(3), Tile::man(4)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::pin(3), Tile::pin(4), Tile::pin(5)],
        ],
    );
    hand.set_concealed(1, yakuhai_tenpai());
    recording_from_hand(
        hand,
        2122,
        1,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn menzen_tsumo() -> MatchRecording {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2124));
    let deal = wall.deal(2).unwrap();
    let mut hand = HandState::from_deal(wall, deal, rules);
    hand.set_concealed(2, winning_tanyao_tiles());
    hand.last_draw = Some(Tile::pin(2));
    hand.is_dealer_first_turn = false;
    recording_from_hand(
        hand,
        2124,
        2,
        meta("", "", &[]),
        Some(vec![Action::Tsumo]),
        0,
        RoundWind::East,
        1,
    )
}

fn chi_left() -> MatchRecording {
    let discarded = Tile::pin(6);
    let hand = start_reaction(
        2125,
        0,
        discarded,
        [
            vec![discarded, Tile::man(1), Tile::man(2)],
            vec![Tile::pin(4), Tile::pin(5), Tile::sou(9)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(7)],
        ],
    );
    recording_from_hand(
        hand,
        2125,
        1,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn chiitoitsu_tiles() -> Vec<Tile> {
    vec![
        Tile::man(2),
        Tile::man(2),
        Tile::man(3),
        Tile::man(3),
        Tile::man(4),
        Tile::man(4),
        Tile::pin(5),
        Tile::pin(5),
        Tile::pin(6),
        Tile::pin(6),
        Tile::sou(7),
        Tile::sou(7),
        Tile::sou(8),
        Tile::sou(8),
    ]
}

fn yakuhai_red_tenpai() -> Vec<Tile> {
    vec![
        Tile::man(2),
        Tile::man(3),
        Tile::man(4),
        Tile::pin(5),
        Tile::pin(6),
        Tile::pin(7),
        Tile::sou(6),
        Tile::sou(7),
        Tile::sou(8),
        Tile::pin(3),
        Tile::pin(3),
        Tile::dragon(Dragon::Red),
        Tile::dragon(Dragon::Red),
    ]
}

fn pinfu_tenpai() -> Vec<Tile> {
    vec![
        Tile::man(1),
        Tile::man(2),
        Tile::man(3),
        Tile::man(5),
        Tile::man(6),
        Tile::man(7),
        Tile::pin(3),
        Tile::pin(4),
        Tile::pin(5),
        Tile::pin(8),
        Tile::pin(8),
        Tile::sou(2),
        Tile::sou(3),
    ]
}

fn riichi_tsumo() -> MatchRecording {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2130));
    let deal = wall.deal(2).unwrap();
    let mut hand = HandState::from_deal(wall, deal, rules);
    hand.set_concealed(2, winning_tanyao_tiles());
    hand.riichi[2] = true;
    hand.last_draw = Some(Tile::pin(2));
    hand.is_dealer_first_turn = false;
    recording_from_hand(
        hand,
        2130,
        2,
        meta("", "", &[]),
        Some(vec![Action::Tsumo]),
        0,
        RoundWind::East,
        1,
    )
}

fn tanyao_ron() -> MatchRecording {
    let discarded = Tile::pin(2);
    let mut hand = start_reaction(
        2131,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(3)],
            vec![Tile::man(2), Tile::man(3), Tile::man(4)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::pin(4), Tile::pin(5), Tile::pin(6)],
        ],
    );
    hand.set_concealed(1, tenpai_waiting_on_p2());
    pass_all_except(&mut hand, 1);
    recording_from_hand(
        hand,
        2131,
        1,
        meta("", "", &[]),
        Some(vec![Action::Ron, Action::Pass]),
        0,
        RoundWind::East,
        1,
    )
}

fn chiitoitsu_tsumo() -> MatchRecording {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2132));
    let deal = wall.deal(0).unwrap();
    let mut hand = HandState::from_deal(wall, deal, rules);
    hand.set_concealed(0, chiitoitsu_tiles());
    hand.last_draw = Some(Tile::sou(8));
    hand.is_dealer_first_turn = false;
    recording_from_hand(
        hand,
        2132,
        0,
        meta("", "", &[]),
        Some(vec![Action::Tsumo]),
        0,
        RoundWind::East,
        1,
    )
}

fn yakuhai_red_ron() -> MatchRecording {
    let discarded = Tile::dragon(Dragon::Red);
    let mut hand = start_reaction(
        2133,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::man(2), Tile::man(3), Tile::man(4)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::pin(3), Tile::pin(4), Tile::pin(5)],
        ],
    );
    hand.set_concealed(3, yakuhai_red_tenpai());
    pass_all_except(&mut hand, 3);
    recording_from_hand(
        hand,
        2133,
        3,
        meta("", "", &[]),
        Some(vec![Action::Ron, Action::Pass]),
        0,
        RoundWind::East,
        1,
    )
}

fn open_tanyao_ron() -> MatchRecording {
    let discarded = Tile::pin(2);
    let mut hand = start_reaction(
        2134,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(3)],
            vec![Tile::man(2), Tile::man(3), Tile::man(4)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::pin(4), Tile::pin(5), Tile::pin(6)],
        ],
    );
    let meld = Meld::pon([Tile::sou(5), Tile::sou(5), Tile::sou(5)], Tile::sou(5)).expect("pon");
    let concealed = vec![
        Tile::man(2),
        Tile::man(3),
        Tile::man(4),
        Tile::pin(3),
        Tile::pin(4),
        Tile::pin(5),
        Tile::pin(6),
        Tile::pin(7),
        Tile::pin(8),
        Tile::pin(2),
    ];
    let h = Hand::new(Concealed::from_tiles(concealed), vec![meld]).expect("open hand");
    hand.set_hand(1, h);
    pass_all_except(&mut hand, 1);
    recording_from_hand(
        hand,
        2134,
        1,
        meta("", "", &[]),
        Some(vec![Action::Ron, Action::Pass]),
        0,
        RoundWind::East,
        1,
    )
}

fn pinfu_ron() -> MatchRecording {
    let discarded = Tile::sou(4);
    let mut hand = start_reaction(
        2135,
        2,
        discarded,
        [
            vec![Tile::man(1), Tile::man(2), Tile::man(3)],
            vec![Tile::pin(1), Tile::pin(2), Tile::pin(3)],
            vec![Tile::pin(4), Tile::pin(5), Tile::pin(6)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
        ],
    );
    hand.set_concealed(0, pinfu_tenpai());
    pass_all_except(&mut hand, 0);
    recording_from_hand(
        hand,
        2135,
        0,
        meta("", "", &[]),
        Some(vec![Action::Ron, Action::Pass]),
        0,
        RoundWind::East,
        1,
    )
}

fn with_wall_kan_count(state: HandState, kan_count: u8) -> HandState {
    let mut snapshot = state.to_snapshot();
    snapshot.wall.kan_count = kan_count;
    snapshot.wall.rinshan_taken = kan_count;
    HandState::from_snapshot(snapshot, state.config().clone()).expect("wall kan snapshot")
}

fn with_dead_wall_tile(state: HandState, index: usize, tile: Tile) -> HandState {
    let mut snapshot = state.to_snapshot();
    snapshot.wall.dead[index] = tile;
    HandState::from_snapshot(snapshot, state.config().clone()).expect("dead wall snapshot")
}

fn non_tenpai_tiles() -> Vec<Tile> {
    vec![
        Tile::man(1),
        Tile::man(1),
        Tile::man(9),
        Tile::man(9),
        Tile::pin(1),
        Tile::pin(9),
        Tile::sou(1),
        Tile::sou(9),
        Tile::wind(Wind::East),
        Tile::wind(Wind::South),
        Tile::wind(Wind::West),
        Tile::wind(Wind::North),
        Tile::dragon(Dragon::White),
    ]
}

fn inject_discard_tile(hand: &mut [Tile], tile: Tile) {
    if let Some(index) = hand.iter().position(|&t| t == tile) {
        hand.swap(0, index);
    } else {
        hand[0] = tile;
    }
}

fn pass_reaction_round(state: &mut HandState) {
    for _ in 0..8 {
        if state.phase() != HandPhase::Reaction {
            break;
        }
        if let Some(seat) = state.pending_reaction_seat() {
            state.apply(seat, Action::Pass).unwrap();
        }
    }
}

fn advance_to_discard_actor(state: &mut HandState, seat: usize) {
    for _ in 0..64 {
        if state.phase() == HandPhase::Discard && state.current_actor() == seat {
            return;
        }
        match state.phase() {
            HandPhase::Draw => {
                state.apply(state.current_actor(), Action::Draw).unwrap();
            }
            HandPhase::Discard => {
                let actor = state.current_actor();
                let tile = state.hand(actor).concealed().tiles()[0];
                state.apply(actor, Action::Discard(tile)).unwrap();
            }
            HandPhase::Reaction => pass_reaction_round(state),
            _ => break,
        }
    }
}

fn second_pin2_ron_after_cleared_furiten(state: &mut HandState, furiten_seat: usize) -> HandState {
    state.apply(furiten_seat, Action::Draw).unwrap();
    let safe = state
        .hand(furiten_seat)
        .concealed()
        .tiles()
        .iter()
        .find(|&&tile| tile != Tile::pin(2))
        .copied()
        .unwrap_or(Tile::man(5));
    state.apply(furiten_seat, Action::Discard(safe)).unwrap();
    pass_reaction_round(state);
    let dealer = (furiten_seat + 3) % 4;
    advance_to_discard_actor(state, dealer);
    state.set_concealed(furiten_seat, tenpai_waiting_on_p2());
    let mut dealer_hand: Vec<Tile> = state.hand(dealer).concealed().tiles().to_vec();
    inject_discard_tile(&mut dealer_hand, Tile::pin(2));
    state.set_concealed(dealer, dealer_hand);
    state.apply(dealer, Action::Discard(Tile::pin(2))).unwrap();
    state.clone()
}

fn chi_middle() -> MatchRecording {
    let discarded = Tile::pin(5);
    let hand = start_reaction(
        2140,
        0,
        discarded,
        [
            vec![discarded, Tile::man(1), Tile::man(2)],
            vec![Tile::pin(4), Tile::pin(6), Tile::sou(9)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(7)],
        ],
    );
    recording_from_hand(
        hand,
        2140,
        1,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn chi_right() -> MatchRecording {
    let discarded = Tile::pin(3);
    let hand = start_reaction(
        2141,
        0,
        discarded,
        [
            vec![discarded, Tile::man(1), Tile::man(2)],
            vec![Tile::pin(4), Tile::pin(5), Tile::sou(9)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::sou(4), Tile::sou(5), Tile::sou(7)],
        ],
    );
    recording_from_hand(
        hand,
        2141,
        1,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn kakan() -> MatchRecording {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2142));
    let deal = wall.deal(0).unwrap();
    let mut hand = HandState::from_deal(wall, deal, rules);
    let pon = Meld::pon([Tile::sou(5), Tile::sou(5), Tile::sou(5)], Tile::sou(5)).unwrap();
    let open_hand = Hand::new(
        Concealed::from_tiles(vec![
            Tile::sou(5),
            Tile::man(2),
            Tile::man(3),
            Tile::man(4),
            Tile::man(6),
            Tile::man(7),
            Tile::man(8),
            Tile::pin(2),
            Tile::pin(3),
            Tile::pin(4),
            Tile::pin(6),
        ]),
        vec![pon],
    )
    .unwrap();
    hand.set_hand(0, open_hand);
    recording_from_hand(
        hand,
        2142,
        0,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn chankan_ron() -> MatchRecording {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2143));
    let deal = wall.deal(1).unwrap();
    let mut hand = HandState::from_deal(wall, deal, rules);
    let pon = Meld::pon([Tile::sou(5), Tile::sou(5), Tile::sou(5)], Tile::sou(5)).unwrap();
    hand.set_hand(
        1,
        Hand::new(
            Concealed::from_tiles(vec![
                Tile::sou(5),
                Tile::man(2),
                Tile::man(3),
                Tile::man(4),
                Tile::man(6),
                Tile::man(7),
                Tile::man(8),
                Tile::pin(2),
                Tile::pin(3),
                Tile::pin(4),
                Tile::pin(6),
            ]),
            vec![pon],
        )
        .unwrap(),
    );
    hand.set_concealed(
        2,
        vec![
            Tile::man(2),
            Tile::man(3),
            Tile::man(4),
            Tile::pin(3),
            Tile::pin(4),
            Tile::pin(5),
            Tile::sou(6),
            Tile::sou(7),
            Tile::sou(8),
            Tile::sou(5),
            Tile::sou(5),
            Tile::sou(9),
            Tile::sou(9),
        ],
    );
    hand
        .apply(1, Action::Kan(KanIntent::Added { meld_index: 0 }))
        .unwrap();
    hand.apply(0, Action::Pass).unwrap();
    hand.apply(3, Action::Pass).unwrap();
    recording_from_hand(
        hand,
        2143,
        2,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn tenpai_waiting_on_p2_alt() -> Vec<Tile> {
    vec![
        Tile::man(4),
        Tile::man(5),
        Tile::man(6),
        Tile::pin(3),
        Tile::pin(4),
        Tile::pin(5),
        Tile::sou(6),
        Tile::sou(7),
        Tile::sou(8),
        Tile::sou(3),
        Tile::sou(3),
        Tile::sou(3),
        Tile::pin(7),
    ]
}

fn tenpai_waiting_on_p2_other() -> Vec<Tile> {
    vec![
        Tile::man(6),
        Tile::man(7),
        Tile::man(8),
        Tile::pin(3),
        Tile::pin(4),
        Tile::pin(5),
        Tile::sou(6),
        Tile::sou(7),
        Tile::sou(8),
        Tile::sou(4),
        Tile::sou(4),
        Tile::sou(4),
        Tile::pin(8),
    ]
}

fn multi_ron_reaction(
    seed: u64,
    tenpai_hands: [Vec<Tile>; 3],
    human_seat: usize,
) -> MatchRecording {
    let discarded = Tile::pin(2);
    let mut hand = start_reaction(
        seed,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(3)],
            vec![Tile::man(1), Tile::man(2), Tile::man(3)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(9)],
            vec![Tile::pin(4), Tile::pin(5), Tile::pin(6)],
        ],
    );
    hand.set_concealed(1, tenpai_hands[0].clone());
    hand.set_concealed(2, tenpai_hands[1].clone());
    hand.set_concealed(3, tenpai_hands[2].clone());
    recording_from_hand(
        hand,
        seed,
        human_seat,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn double_ron() -> MatchRecording {
    let mut recording = multi_ron_reaction(
        2144,
        [
            tenpai_waiting_on_p2(),
            build_hand(2, vec![Tile::man(1), Tile::man(2), Tile::man(3)]),
            tenpai_waiting_on_p2_alt(),
        ],
        1,
    );
    recording.rules_config.double_ron = true;
    recording
}

fn triple_ron() -> MatchRecording {
    let mut recording = multi_ron_reaction(
        2145,
        [
            tenpai_waiting_on_p2(),
            tenpai_waiting_on_p2_alt(),
            tenpai_waiting_on_p2_other(),
        ],
        1,
    );
    recording.rules_config.double_ron = true;
    recording.rules_config.triple_ron = true;
    recording
}

fn pin2_ron_reaction(seed: u64) -> HandState {
    let discarded = Tile::pin(2);
    let mut hand = start_reaction(
        seed,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(3)],
            vec![Tile::man(2), Tile::man(3), Tile::man(4)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::pin(4), Tile::pin(5), Tile::pin(6)],
        ],
    );
    hand.set_concealed(1, tenpai_waiting_on_p2());
    hand
}

fn furiten_temporary() -> MatchRecording {
    let mut hand = pin2_ron_reaction(2146);
    hand.apply(1, Action::Pass).unwrap();
    recording_from_hand(
        hand,
        2146,
        1,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn furiten_cleared() -> MatchRecording {
    let mut hand = pin2_ron_reaction(2147);
    hand.apply(1, Action::Pass).unwrap();
    pass_reaction_round(&mut hand);
    hand = second_pin2_ron_after_cleared_furiten(&mut hand, 1);
    recording_from_hand(
        hand,
        2147,
        1,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn furiten_riichi() -> MatchRecording {
    let mut hand = pin2_ron_reaction(2148);
    hand.riichi[1] = true;
    hand.apply(1, Action::Pass).unwrap();
    pass_reaction_round(&mut hand);
    hand = second_pin2_ron_after_cleared_furiten(&mut hand, 1);
    recording_from_hand(
        hand,
        2148,
        1,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn dora_kan_chain() -> MatchRecording {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2149));
    let deal = wall.deal(1).unwrap();
    let hand = with_wall_kan_count(HandState::from_deal(wall, deal, rules), 2);
    recording_from_hand(
        hand,
        2149,
        1,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn ura_dora_riichi() -> MatchRecording {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2150));
    let deal = wall.deal(2).unwrap();
    let mut hand = HandState::from_deal(wall, deal, rules);
    hand = with_dead_wall_tile(hand, 0, Tile::pin(3));
    hand = with_dead_wall_tile(hand, 1, Tile::pin(3));
    hand.set_concealed(2, winning_tanyao_tiles());
    hand.riichi[2] = true;
    hand.last_draw = Some(Tile::pin(2));
    hand.is_dealer_first_turn = false;
    recording_from_hand(
        hand,
        2150,
        2,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn aka_dora_hand() -> Vec<Tile> {
    vec![
        Tile::red_five(Suit::Man),
        Tile::man(2),
        Tile::man(3),
        Tile::man(4),
        Tile::red_five(Suit::Pin),
        Tile::pin(6),
        Tile::pin(7),
        Tile::pin(8),
        Tile::red_five(Suit::Sou),
        Tile::sou(6),
        Tile::sou(7),
        Tile::sou(8),
        Tile::pin(2),
        Tile::pin(3),
    ]
}

fn aka_dora_on() -> MatchRecording {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2151));
    let deal = wall.deal(0).unwrap();
    let mut hand = HandState::from_deal(wall, deal, rules);
    hand.set_concealed(0, aka_dora_hand());
    recording_from_hand(
        hand,
        2151,
        0,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn aka_dora_off() -> MatchRecording {
    let mut config = RulesConfig::standard();
    config.aka_dora = false;
    let mut wall = Wall::new(&config, StdRng::seed_from_u64(2152));
    let deal = wall.deal(0).unwrap();
    let mut hand = HandState::from_deal(wall, deal, config);
    hand.set_concealed(0, aka_dora_hand());
    recording_from_hand(
        hand,
        2152,
        0,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn mangan_ron() -> MatchRecording {
    let discarded = Tile::sou(8);
    let mut hand = start_reaction(
        2153,
        1,
        discarded,
        [
            vec![Tile::man(1), Tile::man(2), Tile::man(3)],
            vec![discarded, Tile::pin(1), Tile::pin(2)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::pin(4), Tile::pin(5), Tile::pin(6)],
        ],
    );
    let mut winning = chiitoitsu_tiles();
    winning.pop();
    hand.set_concealed(0, winning);
    hand.riichi[0] = true;
    hand.double_riichi[0] = true;
    pass_all_except(&mut hand, 0);
    recording_from_hand(
        hand,
        2153,
        0,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn honba_scoring() -> MatchRecording {
    let discarded = Tile::pin(2);
    let mut hand = start_reaction(
        2154,
        0,
        discarded,
        [
            vec![discarded, Tile::pin(1), Tile::pin(3)],
            vec![Tile::man(2), Tile::man(3), Tile::man(4)],
            vec![Tile::sou(1), Tile::sou(2), Tile::sou(3)],
            vec![Tile::pin(4), Tile::pin(5), Tile::pin(6)],
        ],
    );
    hand.set_honba_for_test(2);
    hand.set_concealed(1, tenpai_waiting_on_p2());
    pass_all_except(&mut hand, 1);
    recording_from_hand(
        hand,
        2154,
        1,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn dealer_adjusted_hand(dealer: usize, seat: usize, tiles: Vec<Tile>) -> Vec<Tile> {
    let mut hand = tiles;
    if seat == dealer && hand.len() == 13 {
        hand.push(Tile::pin(1));
    }
    hand
}

fn exhaustive_draw_mixed() -> MatchRecording {
    let seed = 2155;
    let mut game = Game::new(RulesConfig::standard(), seed).unwrap();
    let dealer = game.dealer();
    let hand = game.hand_mut();
    hand.set_concealed(0, dealer_adjusted_hand(dealer, 0, tenpai_waiting_on_p2()));
    hand.set_concealed(1, dealer_adjusted_hand(dealer, 1, non_tenpai_tiles()));
    hand.set_concealed(2, dealer_adjusted_hand(dealer, 2, tenpai_waiting_on_p2()));
    hand.set_concealed(3, dealer_adjusted_hand(dealer, 3, non_tenpai_tiles()));
    game.hand_mut()
        .play_out_discards(|state, seat| state.hand(seat).concealed().tiles()[0])
        .unwrap();
    capture_match(&game, 0, meta("", "", &[]), None)
}

fn hand_with_wind_tile(seat: usize, dealer: usize, wind: Tile) -> Vec<Tile> {
    let mut tiles = build_hand(seat, vec![wind, Tile::man(2), Tile::man(3)]);
    if seat == dealer {
        tiles.push(Tile::pin(1));
    }
    if !tiles.contains(&wind) {
        tiles[0] = wind;
    }
    tiles
}

fn four_winds_abortive() -> MatchRecording {
    let rules = RulesConfig::standard();
    let east = Tile::wind(Wind::East);
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2156));
    let deal = wall.deal(0).unwrap();
    let mut hand = HandState::from_deal(wall, deal, rules);
    let dealer = hand.dealer();
    for seat in 0..4 {
        hand.set_concealed(seat, hand_with_wind_tile(seat, dealer, east));
    }
    for seat in 0..3 {
        assert_eq!(hand.current_actor(), seat);
        hand.apply(seat, Action::Discard(east)).unwrap();
        pass_reaction_round(&mut hand);
        if seat < 2 {
            hand.apply((seat + 1) % 4, Action::Draw).unwrap();
        } else {
            hand.apply(3, Action::Draw).unwrap();
        }
    }
    hand.apply(3, Action::Discard(east)).unwrap();
    recording_from_hand(
        hand,
        2156,
        3,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn four_kongs_abortive() -> MatchRecording {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2157));
    let deal = wall.deal(0).unwrap();
    let mut hand = with_wall_kan_count(HandState::from_deal(wall, deal, rules), 4);
    hand.phase = HandPhase::Ended;
    hand.end_reason = Some(HandEndReason::AbortiveDraw(AbortiveDrawKind::FourKongs));
    recording_from_hand(
        hand,
        2157,
        0,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn four_riichis_abortive() -> MatchRecording {
    let rules = RulesConfig::standard();
    let mut wall = Wall::new(&rules, StdRng::seed_from_u64(2158));
    let deal = wall.deal(3).unwrap();
    let mut hand = HandState::from_deal(wall, deal, rules);
    for seat in 0..3 {
        hand.riichi[seat] = true;
    }
    hand.set_concealed(3, tenpai_after_draw_p2());
    hand.apply(
        3,
        Action::Riichi {
            discard: Tile::pin(2),
        },
    )
    .unwrap();
    recording_from_hand(
        hand,
        2158,
        3,
        meta("", "", &[]),
        None,
        0,
        RoundWind::East,
        1,
    )
}

fn match_finished() -> MatchRecording {
    let mut recording = south_round();
    recording.seed = 2159;
    recording.match_status = MatchStatus::Finished;
    recording.match_phase = MatchPhase::Ended;
    recording.scores = [32_600, 23_800, 25_200, 23_400];
    recording.kyoku = 4;
    recording.hand_index = 8;
    recording
}
