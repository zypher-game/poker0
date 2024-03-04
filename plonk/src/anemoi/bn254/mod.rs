use crate::anemoi::AnemoiJive;
use ark_bn254::Fr;
use ark_ff::MontFp;

/// The round number of Anemoi.
pub const N_ANEMOI_ROUNDS: usize = 14;

/// The structure that stores the parameters for the Anemoi-Jive hash function for BN254.
pub struct AnemoiJive254;

impl AnemoiJive<Fr, 2usize, N_ANEMOI_ROUNDS> for AnemoiJive254 {
    const ALPHA: u32 = 5u32;
    const GENERATOR: Fr = MontFp!("5");
    const GENERATOR_INV: Fr =
        MontFp!("8755297148735710088898562298102910035419345760166413737479281674630323398247");
    const GENERATOR_SQUARE_PLUS_ONE: Fr = MontFp!("26");
    const ROUND_KEYS_X: [[Fr; 2]; N_ANEMOI_ROUNDS] = [
        [
            MontFp!("37"),
            MontFp!("3751828524803055471428227881618625174556947755988347881191159153764975591158"),
        ],
        [
            MontFp!(
                "13352247125433170118601974521234241686699252132838635793584252509352796067497"
            ),
            MontFp!(
                "21001839722121566863419881512791069124083822968210421491151340238400176843969"
            ),
        ],
        [
            MontFp!("8959866518978803666083663798535154543742217570455117599799616562379347639707"),
            MontFp!(
                "21722442537234642741320951134727484119993387379465291657407115605240150584902"
            ),
        ],
        [
            MontFp!("3222831896788299315979047232033900743869692917288857580060845801753443388885"),
            MontFp!("5574110054747610058729632355948568604793546392090976147435879266833412620404"),
        ],
        [
            MontFp!(
                "11437915391085696126542499325791687418764799800375359697173212755436799377493"
            ),
            MontFp!(
                "19347108854758320361854968987183753113398822331033233961719129079198795045322"
            ),
        ],
        [
            MontFp!(
                "14725846076402186085242174266911981167870784841637418717042290211288365715997"
            ),
            MontFp!(
                "17733032409684964025894538244134113560864261458948810209753406163729963104066"
            ),
        ],
        [
            MontFp!("3625896738440557179745980526949999799504652863693655156640745358188128872126"),
            MontFp!(
                "16641102106808059030810525726117803887885616319153331237086309361060282564245"
            ),
        ],
        [
            MontFp!("463291105983501380924034618222275689104775247665779333141206049632645736639"),
            MontFp!("9245970744804222215259369270991414441925747897718226734085751033703871913242"),
        ],
        [
            MontFp!(
                "17443852951621246980363565040958781632244400021738903729528591709655537559937"
            ),
            MontFp!(
                "18243401795478654990110719981452738859015913555820749188627866268359980949315"
            ),
        ],
        [
            MontFp!(
                "10761214205488034344706216213805155745482379858424137060372633423069634639664"
            ),
            MontFp!(
                "18200337361605220875540054729693479452916227111908726624753615870884702413869"
            ),
        ],
        [
            MontFp!("1555059412520168878870894914371762771431462665764010129192912372490340449901"),
            MontFp!("5239065275003145843160321807696531775964858360555566589197008236687533209496"),
        ],
        [
            MontFp!("7985258549919592662769781896447490440621354347569971700598437766156081995625"),
            MontFp!("9376351072866485300578251734844671764089160611668390200194570180225759013543"),
        ],
        [
            MontFp!("9570976950823929161626934660575939683401710897903342799921775980893943353035"),
            MontFp!("6407880900662180043240104510114613236916437723065414158006054747177494383655"),
        ],
        [
            MontFp!(
                "17962366505931708682321542383646032762931774796150042922562707170594807376009"
            ),
            MontFp!("6245130621382842925623937534683990375669631277871468906941032622563934866013"),
        ],
    ];
    const ROUND_KEYS_Y: [[Fr; 2]; N_ANEMOI_ROUNDS] = [
        [
            MontFp!("8755297148735710088898562298102910035419345760166413737479281674630323398284"),
            MontFp!(
                "16133435893292874812888083849160666046321318009323051176910097996974633748758"
            ),
        ],
        [
            MontFp!("5240474505904316858775051800099222288270827863409873986701694203345984265770"),
            MontFp!(
                "16516377322346822856154252461095180562000423191949949242508439100972699801595"
            ),
        ],
        [
            MontFp!("9012679925958717565787111885188464538194947839997341443807348023221726055342"),
            MontFp!("3513323292129390671339287145562649862242777741759770715956300048086055264273"),
        ],
        [
            MontFp!(
                "21855834035835287540286238525800162342051591799629360593177152465113152235615"
            ),
            MontFp!("5945179541709432313351711573896685950772105367183734375093638912196647730870"),
        ],
        [
            MontFp!(
                "11227229470941648605622822052481187204980748641142847464327016901091886692935"
            ),
            MontFp!("874490282529106871250179638055108647411431264552976943414386206857408624500"),
        ],
        [
            MontFp!("8277823808153992786803029269162651355418392229624501612473854822154276610437"),
            MontFp!(
                "14911320361190879980016686915823914584756893340104182663424627943175208757859"
            ),
        ],
        [
            MontFp!(
                "20904607884889140694334069064199005451741168419308859136555043894134683701950"
            ),
            MontFp!(
                "15657880601171476575713502187548665287918791967520790431542060879010363657805"
            ),
        ],
        [
            MontFp!("1902748146936068574869616392736208205391158973416079524055965306829204527070"),
            MontFp!(
                "14311738005510898661766244714944477794557156116636816483240167459479765463026"
            ),
        ],
        [
            MontFp!(
                "14452570815461138929654743535323908350592751448372202277464697056225242868484"
            ),
            MontFp!(
                "18878429879072656191963192145256996413709289475622337294803628783509021017215"
            ),
        ],
        [
            MontFp!(
                "10548134661912479705005015677785100436776982856523954428067830720054853946467"
            ),
            MontFp!(
                "21613568037783775488400147863112554980555854603176833550688470336449256480025"
            ),
        ],
        [
            MontFp!(
                "17068729307795998980462158858164249718900656779672000551618940554342475266265"
            ),
            MontFp!("2490802518193809975066473675670874471230712567215812226164489400543194289596"),
        ],
        [
            MontFp!(
                "16199718037005378969178070485166950928725365516399196926532630556982133691321"
            ),
            MontFp!(
                "21217120779706380859547833993003263088538196273665904984368420139631145468592"
            ),
        ],
        [
            MontFp!(
                "19148564379197615165212957504107910110246052442686857059768087896511716255278"
            ),
            MontFp!(
                "19611778548789975299387421023085714500105803761017217976092023831374602045251"
            ),
        ],
        [
            MontFp!("5497141763311860520411283868772341077137612389285480008601414949457218086902"),
            MontFp!(
                "19294458970356379238521378434506704614768857764591229894917601756581488831876"
            ),
        ],
    ];
    const PREPROCESSED_ROUND_KEYS_X: [[Fr; 2]; N_ANEMOI_ROUNDS] = [
        [
            MontFp!("9875235397644879082677551174832367614794066768374461301425281161472772669364"),
            MontFp!(
                "21858645442887666000649962444987448281406846313183347319591597416372520936186"
            ),
        ],
        [
            MontFp!(
                "16782726861879113354475406688883165555010923651788393550320367281500279757725"
            ),
            MontFp!(
                "21716573900346641246759819543810812868751270056692513002115190154768694264248"
            ),
        ],
        [
            MontFp!(
                "21061966870155710578694816572762821778055453072317217584284979102445184722013"
            ),
            MontFp!("4549699248331629386178256801656616048620437157601772274159045439897291365892"),
        ],
        [
            MontFp!("2305171087224367456066076264392784488317998917489739225874252016996953925819"),
            MontFp!("753423837773794493583073069146671816131950370427940330023762710613130114284"),
        ],
        [
            MontFp!(
                "10698642504920643104042109480794649581252624576357861152208736776268689349977"
            ),
            MontFp!(
                "15861688217704403960557684449792840983848058583334483774016142194529689550996"
            ),
        ],
        [
            MontFp!("423541189543553676504695293714968243652665955997218133034093874913252335294"),
            MontFp!("2085108831563138776622770735491169518313516126676144009754728151028847503805"),
        ],
        [
            MontFp!("9296577236668306825777791135277154873361365777577262330385210251177355111236"),
            MontFp!(
                "13116726794489691995858453211791287240125772343837695887860897125838790404152"
            ),
        ],
        [
            MontFp!("7770554041004090339324059898697839176238158454150868392168895497050756225989"),
            MontFp!(
                "15470974097886414257515842937693682793336718238774806265675536486339112492265"
            ),
        ],
        [
            MontFp!("2027607608657541590621559284719638781643880713344167215688101555484801034021"),
            MontFp!(
                "17300926706027606418491168431313029799747253325902468045764528997217592945985"
            ),
        ],
        [
            MontFp!(
                "15221128912303148791281337033495862121987426916739733934930599252800117682974"
            ),
            MontFp!(
                "13613239587442289301259781578364994509804749661959905633737037792945804710990"
            ),
        ],
        [
            MontFp!(
                "12005763401209949733243841400657315836955319000163295072425874164656732641981"
            ),
            MontFp!("22705376494938444016386495814792101413510899935471717349511104697970912694"),
        ],
        [
            MontFp!(
                "14955552946683837036542615852864240457364059063741425926890312579929291591324"
            ),
            MontFp!("288970556961158641815624492891121300982160056362935242882312884201067196942"),
        ],
        [
            MontFp!("1581177506494232880889471258607800990368585397168021361552725068313737403708"),
            MontFp!(
                "17981970841153864433894117072118866447373490474241757839845618545859583545511"
            ),
        ],
        [
            MontFp!(
                "13826749781571877127981823420956728751726595679537198790271191951241691704832"
            ),
            MontFp!(
                "21456423298419106344829058807152140321464760141466308742744827391066120856237"
            ),
        ],
    ];
    const PREPROCESSED_ROUND_KEYS_Y: [[Fr; 2]; N_ANEMOI_ROUNDS] = [
        [
            MontFp!(
                "13004335645468876947782817511996516830557692388848756239167689579223703209154"
            ),
            MontFp!(
                "11864075285365324632501660503932294097119662259150439783414276164786389548361"
            ),
        ],
        [
            MontFp!("7862495485034485030006053329979953690634378679977822019470434513025641948468"),
            MontFp!(
                "21778305909519758427732388472275509688429815430677336887808798633722753860845"
            ),
        ],
        [
            MontFp!(
                "12931102024200069317238425826876622077088120606615813415940805446744126635881"
            ),
            MontFp!("7837661096836606004314569173269958689435480650877562806125674115879275837181"),
        ],
        [
            MontFp!(
                "14988274660376568290931678743130590897577302840578069596030418254228064426148"
            ),
            MontFp!(
                "14818345905108638164688641616372585080538194792946547345972306256783653004291"
            ),
        ],
        [
            MontFp!(
                "11966397199239721279456793945370572038235535122896503364930899557716957223959"
            ),
            MontFp!("2853352834541474475776137785488700155363788984994460875907827022572233875584"),
        ],
        [
            MontFp!("6473747423912923573021858532418794714202395821695920085715793777854113577052"),
            MontFp!(
                "14603107593725024233314048684876188310197903996220843563409821502003190608529"
            ),
        ],
        [
            MontFp!(
                "10018141451544555380964804958767235988622088919781088363105734833991047400353"
            ),
            MontFp!("83445762062875740982996603123888928543771735703494814377210678846969285492"),
        ],
        [
            MontFp!("4853894954678028326595990416033041454601372518726024075995342652050367914374"),
            MontFp!(
                "13529950793291157200862531998635554831775405064348392294420225414209107516565"
            ),
        ],
        [
            MontFp!("2807960038839395770936423062783538297061734914581689261511199436908401205594"),
            MontFp!("2959287061458222329954767340179788517820610776269329086252152135975612854535"),
        ],
        [
            MontFp!("1011199386146110957860470152252409466117369100436101125582703221610204956433"),
            MontFp!(
                "11916226082408980147601015423483352131731691070197152337036758512414772646884"
            ),
        ],
        [
            MontFp!("6143620485513326860817743193060169274247928932037486340946124795304534640217"),
            MontFp!("9249411266687228683218385140647077689008430999582930158150164394401064685612"),
        ],
        [
            MontFp!("3865024776110368315374386772707941373393630458661571912715432285796031519218"),
            MontFp!("1124707246745155402135444593036779382685949620543378002907953793036433309720"),
        ],
        [
            MontFp!("3747281796037953947554825537973345299481414684769676458997083724683939123632"),
            MontFp!("516368516371014501734378213574699667472834788768584825363152893957289265859"),
        ],
        [
            MontFp!("8415215912404504262033404854405294287543393294861880020399730040978826989992"),
            MontFp!(
                "10041866203038674311709434184968252713427481340634280330144689403763670911641"
            ),
        ],
    ];
    const MDS_MATRIX: [[Fr; 2]; 2] = [[MontFp!("1"), MontFp!("5")], [MontFp!("5"), MontFp!("26")]];

    fn get_alpha_inv() -> Vec<u64> {
        vec![
            14981214993055009997u64,
            6006880321387387405u64,
            10624953561019755799u64,
            2789598613442376532u64,
        ]
    }
}
