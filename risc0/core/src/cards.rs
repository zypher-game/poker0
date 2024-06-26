use ark_ec::{CurveGroup, PrimeGroup};
use hashbrown::HashMap;

use ark_ed_on_bn254::{EdwardsAffine, EdwardsProjective};
use ark_ff::MontFp;
use rand_chacha::rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use uzkge::{
    anemoi::AnemoiJive254,
    chaum_pedersen::dl::{prove0, verify0, ChaumPedersenDLParameters},
    utils::serialization::{ark_deserialize, ark_serialize},
};

use crate::{
    errors::PokerError,
    schnorr::{KeyPair, PublicKey},
    CiphertextAffineRepr,
};
use crate::{errors::Result, RevealProof};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Suite {
    Club,
    Diamond,
    Heart,
    Spade,
}

impl Suite {
    pub const SUITES: [Suite; 4] = [Suite::Club, Suite::Diamond, Suite::Heart, Suite::Spade];

    pub fn to_u8(&self) -> u8 {
        match self {
            Suite::Club => 0,
            Suite::Diamond => 1,
            Suite::Heart => 2,
            Suite::Spade => 3,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Value {
    pub const VALUES: [Value; 13] = [
        Value::Three,
        Value::Four,
        Value::Five,
        Value::Six,
        Value::Seven,
        Value::Eight,
        Value::Nine,
        Value::Ten,
        Value::Jack,
        Value::Queen,
        Value::King,
        Value::Ace,
        Value::Two,
    ];

    pub fn weight(&self) -> u8 {
        match self {
            Value::Three => 3,
            Value::Four => 4,
            Value::Five => 5,
            Value::Six => 6,
            Value::Seven => 7,
            Value::Eight => 8,
            Value::Nine => 9,
            Value::Ten => 10,
            Value::Jack => 11,
            Value::Queen => 12,
            Value::King => 13,
            Value::Ace => 14,
            Value::Two => 20, // on purpose
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Value::Ace => 1,
            Value::Two => 2,
            Value::Three => 3,
            Value::Four => 4,
            Value::Five => 5,
            Value::Six => 6,
            Value::Seven => 7,
            Value::Eight => 8,
            Value::Nine => 9,
            Value::Ten => 10,
            Value::Jack => 11,
            Value::Queen => 12,
            Value::King => 13,
        }
    }
}

impl PartialOrd for Value {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.weight().partial_cmp(&other.weight())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassicCard {
    value: Value,
    suite: Suite,
}

impl ClassicCard {
    pub fn new(value: Value, suite: Suite) -> Self {
        Self { value, suite }
    }

    #[inline]
    pub fn get_value(&self) -> Value {
        self.value
    }

    #[inline]
    pub fn weight(&self) -> u8 {
        self.value.weight()
    }

    #[inline]
    pub fn to_bytes(&self) -> Vec<u8> {
        vec![self.value.to_u8(), self.suite.to_u8()]
    }
}

impl std::fmt::Debug for ClassicCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let suite = match self.suite {
            Suite::Club => "♣",
            Suite::Diamond => "♦",
            Suite::Heart => "♥",
            Suite::Spade => "♠",
        };

        let val = match self.value {
            Value::Two => "2",
            Value::Three => "3",
            Value::Four => "4",
            Value::Five => "5",
            Value::Six => "6",
            Value::Seven => "7",
            Value::Eight => "8",
            Value::Nine => "9",
            Value::Ten => "10",
            Value::Jack => "J",
            Value::Queen => "Q",
            Value::King => "K",
            Value::Ace => "A",
        };

        write!(f, "{}{}", suite, val)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub struct EncodingCard(
    #[serde(serialize_with = "ark_serialize", deserialize_with = "ark_deserialize")]
    pub  EdwardsAffine,
);

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Default)]
pub struct RevealCard(
    #[serde(serialize_with = "ark_serialize", deserialize_with = "ark_deserialize")]
    pub  EdwardsAffine,
);

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Default)]
pub struct CryptoCard(pub CiphertextAffineRepr);

impl CryptoCard {
    pub fn rand<R: CryptoRng + RngCore>(prng: &mut R) -> Self {
        Self(CiphertextAffineRepr::rand(prng))
    }
}

#[inline]
pub fn unmask(masked_card: &CiphertextAffineRepr, reveal_cards: &[RevealCard]) -> EncodingCard {
    let aggregate: EdwardsAffine = reveal_cards
        .iter()
        .map(|x| x.0)
        .sum::<EdwardsProjective>()
        .into();

    EncodingCard((masked_card.e2 - aggregate).into())
}

// The zk-friendly reveal algorithm.
pub fn reveal0<R: CryptoRng + RngCore>(
    prng: &mut R,
    keypair: &KeyPair,
    masked_card: &CiphertextAffineRepr,
) -> Result<(RevealCard, RevealProof)> {
    let reveal = masked_card.e1 * keypair.private_key.0;

    let parameters = ChaumPedersenDLParameters {
        g: masked_card.e1.into(),
        h: EdwardsProjective::generator(),
    };

    let proof = prove0::<_, AnemoiJive254>(
        prng,
        &parameters,
        &keypair.private_key.0,
        &reveal,
        &keypair.public_key.get_raw(),
    )
    .map_err(|_| PokerError::ReVealError)?;

    Ok((RevealCard(reveal.into_affine()), proof))
}

// The zk-friendly verify reveal algorithm.
pub fn verify_reveal0(
    pk: &PublicKey,
    masked_card: &CiphertextAffineRepr,
    reveal_card: &RevealCard,
    proof: &RevealProof,
) -> Result<()> {
    let parameters = ChaumPedersenDLParameters {
        g: masked_card.e1.into(),
        h: EdwardsProjective::generator(),
    };

    verify0::<AnemoiJive254>(&parameters, &reveal_card.0.into(), &pk.0.into(), proof)
        .map_err(|_| PokerError::VerifyReVealError)
}

lazy_static! {
    pub static ref DECK: Vec<ClassicCard> = {
        let mut deck = vec![];

        for value in Value::VALUES.iter() {
            match value {
                Value::Ace => {
                    for suite in Suite::SUITES.iter().skip(1) {
                        let classic_card = ClassicCard::new(*value, *suite);
                        deck.push(classic_card);
                    }
                }

                Value::Two => {
                    let classic_card = ClassicCard::new(*value, Suite::Heart);
                    deck.push(classic_card);
                }

                _ => {
                    for suite in Suite::SUITES.iter() {
                        let classic_card = ClassicCard::new(*value, *suite);
                        deck.push(classic_card);
                    }
                }
            }
        }

        deck
    };
    pub static ref ENCODING_CARDS_MAPPING: HashMap<EdwardsAffine, ClassicCard> = {
        let point = vec![
            (
                MontFp!(
                    "6026298923809268772437858225988782659101076347507542676032862122004677189994"
                ),
                MontFp!(
                    "6555228313439691250417954674116901492949427813030610178881073802325970545253"
                ),
            ),
            (
                MontFp!(
                    "16362293428140612904060719188743223270015331302127350151220370086038401177449"
                ),
                MontFp!(
                    "20563566001184911091227008957639804539026084696569228734513535014833098836905"
                ),
            ),
            (
                MontFp!(
                    "15389457865468001090209016072255485768688374340165589907522575062317428259617"
                ),
                MontFp!(
                    "8885496021105989521488829304035767854967833651308356349527834032638357646204"
                ),
            ),
            (
                MontFp!(
                    "14674519018196393776212834424165276464808848932598378492354476210551540545680"
                ),
                MontFp!(
                    "1209996045626851132343281431245070906025951842792598041308089842842516750314"
                ),
            ),
            (
                MontFp!(
                    "6237041078346503160390732816505586396471874203042905457656094267379968068878"
                ),
                MontFp!(
                    "10850948752386193101680562141930714702944852903793442995664489687906658205602"
                ),
            ),
            (
                MontFp!(
                    "7023275951231395860395622090696513982498714789158277194981243353353479551188"
                ),
                MontFp!(
                    "7554044650472055637082146583357381943810549668090794301958455937771984853653"
                ),
            ),
            (
                MontFp!(
                    "6337361459696085339872539934974192707621028108330146406055022104087508557722"
                ),
                MontFp!(
                    "19185189884970557385993114543765936959143423648086164217864648061973914736585"
                ),
            ),
            (
                MontFp!(
                    "11604749113171227047397430928826174427883495802757155706375499556717278962649"
                ),
                MontFp!(
                    "17979247045178308120297611147031155293136188238955418693740119049543922273986"
                ),
            ),
            (
                MontFp!(
                    "252527253864790852258847454738046274114199923695965035159313272505685874356"
                ),
                MontFp!(
                    "17258470332111850749910237717646927543908736330171432963716812914847749306282"
                ),
            ),
            (
                MontFp!(
                    "13298423324217002745023084292837522280580422852028059794939127341889568821965"
                ),
                MontFp!(
                    "6660323797885233754759271104980348870453413748427624639537434033676351940951"
                ),
            ),
            (
                MontFp!(
                    "10839188804377270249944512788726928929025072951812989866086418652199635584905"
                ),
                MontFp!(
                    "8263658308572433101428103993245178455560486601396379762846491496323088562551"
                ),
            ),
            (
                MontFp!(
                    "5327458058654049508556397136583161942138851180748156071086790317868265171208"
                ),
                MontFp!(
                    "27502148200461933629352153888914612930836539021878741812919348882467426215"
                ),
            ),
            (
                MontFp!(
                    "5843018513367319817375442741247712187668299964039809429721488444311683897933"
                ),
                MontFp!(
                    "20165887190824145722629305236973749887397579862591945742085201481080654962872"
                ),
            ),
            (
                MontFp!(
                    "15097249171194748201821757689477668190271945435062435069501513183678223875190"
                ),
                MontFp!(
                    "20486817381145387088056624523044944601959130733382799257722067150391080766952"
                ),
            ),
            (
                MontFp!(
                    "12949214038414260046942149319764528771012363781631164662812481569791382158008"
                ),
                MontFp!(
                    "16265168704690202799236147225479242655969706536969516341991887335653182663309"
                ),
            ),
            (
                MontFp!(
                    "2260964029247825280169068960060870228965146670006714076137368312026911691507"
                ),
                MontFp!(
                    "1963913215765432276726350490626550759275859232087632625923843810629184148287"
                ),
            ),
            (
                MontFp!(
                    "1342466192530232894361334480036160030956618581460423544638722220426676414711"
                ),
                MontFp!(
                    "11228725700846584466344859339324447993579304850622403775824737343691675039267"
                ),
            ),
            (
                MontFp!(
                    "3747163095390652652659648808723715077408727807672528667735119621148260751338"
                ),
                MontFp!(
                    "13442521717818375039928425416388987205182894997321122423922898650093984967176"
                ),
            ),
            (
                MontFp!(
                    "3422244759235674236283161818035080627810170762480420554975355085183944396734"
                ),
                MontFp!(
                    "10700856697397793713843370655275368951827607233026085074058850819114212032114"
                ),
            ),
            (
                MontFp!(
                    "20122041339382464925260010365434964646201347288990248132434891811276920274653"
                ),
                MontFp!(
                    "9035759272377843003980886756349034294824503344172626063390893518330673326307"
                ),
            ),
            (
                MontFp!(
                    "16053938530311191236068405774243936295538100644065375123780424078826208916772"
                ),
                MontFp!(
                    "10327526455032677953043506100412473682656929351030184438134964416392047981134"
                ),
            ),
            (
                MontFp!(
                    "16003211653725402648247923887433285138372785133653684026290624150550136560440"
                ),
                MontFp!(
                    "9084243849402393456726062775407634938452735541768390149067120542748250011789"
                ),
            ),
            (
                MontFp!(
                    "15285830087694757972321005870992515579030694483853443382943183483515570500037"
                ),
                MontFp!(
                    "4254329505116654337734191547842673887721455378703849426082195224760002925266"
                ),
            ),
            (
                MontFp!(
                    "16771619335458850458241592716500296371004657739240600188294332646841228659450"
                ),
                MontFp!(
                    "125692011848353708416847364038245201661929390256023204098173070969518989099"
                ),
            ),
            (
                MontFp!(
                    "14080578526292176157375972709537522231200285961749313944056512879118654502481"
                ),
                MontFp!(
                    "6844821315115673042446648661829515451072273815567574380256234912317288069992"
                ),
            ),
            (
                MontFp!(
                    "19600334536210785779686224943053755507370739328047478757636141767884270026286"
                ),
                MontFp!(
                    "2151944498283477129764465039087255500107280150832928596645088161698005825228"
                ),
            ),
            (
                MontFp!(
                    "8620469181557679931895280994878877334850450380702792685530214159280946567669"
                ),
                MontFp!(
                    "10223801344033958300405239671012626171048609908992221325845490281850372649734"
                ),
            ),
            (
                MontFp!(
                    "8195646654677735837945975774323302332781054502578401605869722985099579213370"
                ),
                MontFp!(
                    "10639211679232663375381790410627770876501078679157661906844335745934938932538"
                ),
            ),
            (
                MontFp!(
                    "18992615796646295222731184387205916327788637840910761274055673950443180392132"
                ),
                MontFp!(
                    "16762687479777443953207927463239607550052406515525297991111827136474486077640"
                ),
            ),
            (
                MontFp!(
                    "11620529016589879573038601763339274022438433482046842760375335100133075160666"
                ),
                MontFp!(
                    "1059015883336193513133066425172778790417044812276501998555632661069847847692"
                ),
            ),
            (
                MontFp!(
                    "11475102161821149019093722791389829727971574610468562236290339451521039028979"
                ),
                MontFp!(
                    "4869231239414537408256953211775704771016398904360604601363875939129242817303"
                ),
            ),
            (
                MontFp!(
                    "12651808384931669401143929833875362409329668175837415471606279689121150021508"
                ),
                MontFp!(
                    "13418290858812863723831391685236754707725401371823830852725019628952989495703"
                ),
            ),
            (
                MontFp!(
                    "18962420875658323319893637550111357150582749656235294465414329067810818003301"
                ),
                MontFp!(
                    "10815724698969126109285229625437440380850676559689011559142634193417758607102"
                ),
            ),
            (
                MontFp!(
                    "1471555874553510146008284815121941329783238957571763300623356704304089523811"
                ),
                MontFp!(
                    "13482558219036907743250450556130242935371067828842793959532735638635738427732"
                ),
            ),
            (
                MontFp!(
                    "1663109298670263953807706365498924569752940979731353627667552102054212996252"
                ),
                MontFp!(
                    "21338016212915064326508033889408811675201359899715538209576992468162551939035"
                ),
            ),
            (
                MontFp!(
                    "21770716502265732917881009538007827323070125646239208898929502725819466546693"
                ),
                MontFp!(
                    "20183910633538028458060649512546072084758299362218270367146487408795794636222"
                ),
            ),
            (
                MontFp!(
                    "12445469636638631885868859134374778398967799382774337855274089786935412196959"
                ),
                MontFp!(
                    "3416167175581597796021493145414015654959870458845056488541593084938979562958"
                ),
            ),
            (
                MontFp!(
                    "1751467210247904608682948132059551467923064963614542922361689255325919955138"
                ),
                MontFp!(
                    "2388826957941719791635323056631119723429091303055383725028883220349334828970"
                ),
            ),
            (
                MontFp!(
                    "14129585364251110359077446627310530513585885072513317202367324216674474791459"
                ),
                MontFp!(
                    "529012615332326422726327645696717876412013385613085711221614321197578149005"
                ),
            ),
            (
                MontFp!(
                    "5231861596547221745955305221113629932497921393706877215156444794500279333231"
                ),
                MontFp!(
                    "12093935025703570083591881042292379852606298962381866964042819317998775663453"
                ),
            ),
            (
                MontFp!(
                    "10054203053434407875855934003406511095483759682905943161173353579297650337598"
                ),
                MontFp!(
                    "21586410790690100172846730616598282162391822404166351890724057571284029080780"
                ),
            ),
            (
                MontFp!(
                    "21296605669697269167286039450320318073399828950648628070772101315745223556190"
                ),
                MontFp!(
                    "10513624388825413555914292213598092905400311911129144125474842424972774685811"
                ),
            ),
            (
                MontFp!(
                    "12926050015961349251904862356880485489534144491831744225328952070581352369648"
                ),
                MontFp!(
                    "979864346933346274979171426534228337146633624006652533371104773506221331215"
                ),
            ),
            (
                MontFp!(
                    "14318181171015104167809841279200565902842957493417443056154095977735081681484"
                ),
                MontFp!(
                    "444814435282239009311597817237417803841177317330280006074808076159074540140"
                ),
            ),
            (
                MontFp!(
                    "2122143479993534869411462200078178614379667249936817849471162222188463634461"
                ),
                MontFp!(
                    "13297268678195587927537583683620354785663767440718335943938081953224598364091"
                ),
            ),
            (
                MontFp!(
                    "20389739020263383556773117709883547364828268496255770653347824091673289864103"
                ),
                MontFp!(
                    "15585708677576873723004786813205839508617156875533004481404463596792402675335"
                ),
            ),
            (
                MontFp!(
                    "3547989487119640962145364135489613804371251082314354217281875682296509404957"
                ),
                MontFp!(
                    "7662994551645956188354044474383084992629453090384886249634574674457378950674"
                ),
            ),
            (
                MontFp!(
                    "16117840592296819892588371144429944465770729876804492216309519369032459599132"
                ),
                MontFp!(
                    "10639437100116784414849112948670820976502503848836580728911458252003366472114"
                ),
            ),
        ];

        let encoding_card = point
            .into_iter()
            .map(|(x, y)| EdwardsAffine::new_unchecked(x, y))
            .collect::<Vec<EdwardsAffine>>();

        let mut map = HashMap::new();
        for (i, classic_card) in DECK.iter().enumerate() {
            map.insert(encoding_card[i], *classic_card);
        }

        map
    };
}
