use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize, de};

#[repr(u16)]
#[derive(Debug, Clone, Copy, Deserialize, TryFromPrimitive)]
pub enum SocietyCode {
    Dp = 0,              // Public Domain
    Acum = 1,            // Israel
    Addaf = 2,           // Brazil
    Aepi = 3,            // Greece
    Agadu = 4,           // Uruguay
    Akm = 5,             // Austria
    Bucuda = 6,          // Central Afr Rep
    Apdayc = 7,          // Peru
    Apra = 8,            // Australia
    Artisjus = 9,        // Hungary
    Ascap = 10,          // USA
    AustroMechana = 11,  // Austria
    Amcos = 12,          // Australia
    Awa = 13,            // German Dem Rep
    Argentores = 14,     // Argentina
    Apa = 15,            // Paraguay
    Bumda = 16,          // Mali
    Amra = 17,           // USA
    Bgda = 18,           // Guinea
    Bmda = 19,           // Morocco
    Sodrac = 20,         // Canada
    Bmi = 21,            // USA
    Mcsn = 22,           // Nigeria
    Buma = 23,           // Netherlands
    Burida = 24,         // Ivory Coast
    Bsda = 25,           // Senegal
    Cash = 26,           // Hong Kong
    Capac = 27,          // Canada
    Lita = 28,           // Slovakia
    Scd = 29,            // Chile
    Amar = 30,           // Brazil
    Dilia = 31,          // Czech Republic
    Filscap = 32,        // Philippines
    Omda = 33,           // Madagascar
    HarryFoxAgency = 34, // USA
    Gema = 35,           // German Fed Rep
    Iprs = 36,           // India
    Bubedra = 37,        // Benin
    Jasrac = 38,         // Japan
    Musicautor = 39,     // Bulgaria
    Koda = 40,           // Denmark
    LiterarMechana = 41, // Austria
    Lvg = 42,            // Austria
    Mcsk = 43,           // Kenya
    Mcps = 44,           // United Kingdom
    Bbda = 45,           // Burkina Faso
    Mrs = 46,            // United Kingdom
    Bcda = 47,           // Congo
    Ncb = 48,            // Scandinavia
    Onda = 49,           // Algeria
    Osa = 50,            // Czech Republic
    Prolitteris = 51,    // Switzerland
    Prs = 52,            // United Kingdom
    Procan = 53,         // Canada
    Alcs = 54,           // United Kingdom
    Sabam = 55,          // Belgium
    Sacd = 56,           // France
    Sacerau = 57,        // Egypt
    Sacem = 58,          // France
    Sacm = 59,           // Mexico
    Sacven = 60,         // Venezuela
    Sadaic = 61,         // Argentina
    Sadembra = 62,       // Brazil
    Samro = 63,          // South Africa
    Sokoj = 64,          // Yugoslavia
    Sayce = 65,          // Ecuador
    Sbacem = 66,         // Brazil
    Sbat = 67,           // Brazil
    /// (SDRM should no longer be used.  
    /// Please use SACEM 058 instead)
    Sdrm = 68, // France
    Spa = 69,            // Portugal
    Sogem = 70,          // Mexico
    Sesac = 71,          // USA
    Sgae = 72,           // Spain
    Scam = 73,           // France
    Siae = 74,           // Italy
    Suissimage = 75,     // Switzerland
    Acemla = 76,         // Puerto Rico
    Stef = 77,           // Iceland
    Stemra = 78,         // Netherlands
    Stim = 79,           // Sweden
    Suisa = 80,          // Switzerland
    Sarral = 81,         // South Africa
    Otpda = 82,          // Tunisia
    Soneca = 83,         // Zaire
    Sayco = 84,          // Columbia
    Soza = 85,           // Slovakia
    Sicam = 86,          // Brazil
    Spacemf = 87,        // French Polynesia
    Cmrra = 88,          // Canada
    Teosto = 89,         // Finland
    Tono = 90,           // Norway
    Ssa = 91,            // Switzerland
    Socinada = 92,       // Cameroon Rep
    Ubc = 93,            // Brazil
    Rao = 94,            // Russia
    Vgwort = 95,         // 95 German
    Cott = 96,           // Trinidad & Tobago
    Zaiks = 97,          // Poland
    Zimra = 98,          // Zimbabwe
    Ns = 99,             //
    Socan = 101,         // Canada
    Nascam = 102,        // Namibia
    Acdam = 103,         // Cuba
    Macp = 104,          // Malaysia
    Masa = 105,          // Mauritius
    Compass = 106,       // Singapore
    Acam = 107,          // Costa Rica
    Cha = 108,           // Taiwan
    Kci = 109,           // Indonesia
    LatgaA = 110,        // Lithuania
    HdsZamp = 111,       // Croatia
    Sazas = 112,         // Slovenia
    Laa = 113,           // Latvia
    Agayc = 114,         // Guatemala
    UcmrAda = 115,       // Romania
    Eau = 116,           // Estonia
    Mesam = 117,         // Turkey
    Komca = 118,         // South Korea
    Mcsc = 119,          // China
    Lira = 120,          // Netherlands
    Vdfs = 121,          // Austria
    AkkaLaa = 122,       // Latvia
    Cosga = 123,         // Ghana
    Cosoma = 124,        // Malawi
    Bnda = 125,          // Niger
    Mct = 126,           // Thailand
    Albautor = 127,      // Albania
    Imro = 128,          // Ireland
    Sobodaycom = 129,    // Bolivia
    Butodra = 130,       // Togo
    Sada = 131,          // Greece
    BildKunst = 132,     // German Federal Republic
    Zamcops = 133,       // Zambia
    Slprs = 134,         // Sri Lanka
    Sadh = 135,          // Greece
    ZampMacedonia = 136, // Macedonia
    Sofam = 137,         // Belgium
    Kopiosto = 138,      // Finland
    CopyDanBilledkunst = 139, // 139 Denmark
    Uacrr = 140,         // Ukraine
    AtnLegacy = 141,     // Chile (formerly known as GESATCH)
    Dalro = 142,         // South Africa
    Teaterautor = 143,   // Bulgaria
    Haa = 144,           // Croatia
    Dprs = 145,          // United Kingdom
    Spac = 146,          // Panama
    Filmautor = 147,     // Bulgaria
    Adagp = 148,         // France
    Ars = 149,           // USA
    Beeldrecht = 150,    // Netherlands
    Bono = 151,          // Norway
    Bus = 152,           // Sweden
    Dacs = 153,          // United Kingdom
    Hungart = 154,       // Hungary
    Somaap = 155,        // Mexico
    Vaga = 156,          // USA
    Vbk = 157,           // Austria
    Vegap = 158,         // Spain
    Viscopy = 159,       // Australia
    Rupis = 160,         // Belarus
    Must = 161,          // Taiwan (Province Of China)
    Ampal = 162,         // Australia
    ApgJapan = 163,      // Japan
    Apsav = 164,         // Peru
    Atn = 165,           // Chile
    Autorarte = 166,     // Venezuela
    Burafo = 167,        // Netherlands
    Cal = 168,           // Australia
    Coscap = 169,        // Barbados
    Cpsn = 170,          // Nepal
    Creaimagen = 171,    // Chile
    Dga = 172,           // United States
    Directores = 173,    // Mexico
    FlimJus = 174,       // 174 Hungary
    Copyro = 175,        // Romania
    Jacap = 176,         // Jamaica
    Kazak = 177,         // Kazakhstan
    Kosa = 178,          // Korea Republic Of
    Kuvasto = 179,       // Finland
    Musikedition = 180,  // Austria
    Nmpa = 181,          // United States
    Pappri = 182,        // Indonesia
    Sack = 183,          // Korea Republic Of
    Sartec = 184,        // Canada
    Sesam = 185,         // France
    Sgdl = 186,          // France
    Snac = 187,          // France
    SocieteDeLimage = 188, // France
    Socinpro = 189,      // Brazil
    Sope = 190,          // Greece
    Spacq = 191,         // Canada
    Sff = 192,           // Sweden
    TheSocietyOfAuthors = 193, //  United Kingdom
    UfficioLegale = 194, // 194 Holy See (Vatican City State)
    Vevam = 195,         // Netherlands
    Wga = 196,           // United States
    Wgj = 197,           // Japan
    ZampSlovenia = 198,  // Slovenia
    Zapa = 199,          // Poland
    Msg = 200,           // Turkey
    Abramus = 201,       // Brazil
    Asdac = 202,         // Moldova _ Republic Of
    Awgacs = 203,        // Australia
    Sas = 204,           // Georgia
    Sodart = 205,        // Canada
    SuomenKirjailijaliitto = 206, // Finland
    TheAuthorsRegistryInc = 207, // United States
    Sga = 208,           // Guinea_Bissau
    Armauthor = 209,     // Armenia
    /// (Formerly CANCOPY)
    Access = 210, // Canada
    Cscs = 212,          // Canada
    Drcc = 213,          // Canada
    Hms = 214,           // Saint Lucia
    Kyrgyzpatent = 215,  // Kyrgyzstan
    Sqn = 216,           // Bosnia And Herzegovina
    Abrac = 217,         // Brazil
    Anacim = 218,        // Brazil
    Assim = 219,         // Brazil
    Atida = 220,         // Brazil
    Sabem = 221,         // Brazil
    Fonoperu = 222,      // Peru
    Cosota = 223,        // Tanzania, United Republic Of
    Somas = 224,         // Mozambique
    Saif = 225,          // France
    Aacimh = 226,        // Honduras
    Sgacedom = 227,      // Dominican Republic
    Roms = 228,          // Russian Federation
    Icg = 229,           // United States
    Adavis = 230,        // Cuba
    Autvis = 231,        // Brazil
    Gestor = 232,        // Czech Republic
    Sacemluxembourg = 233, // Luxembourg
    Ucoso = 234,         // Uganda
    Sacenc = 235,        // France
    WidCentre = 300,     // United States
    Gesac = 301,         // Belgium
    Latinautor = 302,    // Uruguay
    NordDoc = 303,       // Sweden
    Songcode = 304,      // United States
    Imjv = 305,          // Netherlands
    Ccl = 306,           // Trinidad And Tobago
    Misasia = 307,       // Singapore
    Ecad = 308,          // Brazil
    Latinnet = 309,      // Spain
    Diva = 310,          // Hong Kong));
    MusicMark = 707,     // USA));
}

#[derive(Debug, Clone, Copy)]
pub enum TerritoryCode {
    Region(RegionCode),
    Country(CountryCode),
}
impl TerritoryCode {
    pub fn code(&self) -> u16 {
        match self {
            TerritoryCode::Region(x) => *x as u16,
            TerritoryCode::Country(x) => *x as u16,
        }
    }
}

impl Serialize for TerritoryCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let num = match self {
            TerritoryCode::Region(code) => *code as u16,
            TerritoryCode::Country(code) => *code as u16,
        };
        serializer.serialize_u16(num)
    }
}

impl<'de> Deserialize<'de> for TerritoryCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let num = u16::deserialize(deserializer)?;
        if let Ok(code) = CountryCode::try_from(num) {
            return Ok(TerritoryCode::Country(code));
        }
        if let Ok(code) = RegionCode::try_from(num) {
            return Ok(TerritoryCode::Region(code));
        }
        Err(de::Error::custom(format!("invalid TIS code {num}")))
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, Deserialize, TryFromPrimitive)]
pub enum CountryCode {
    Afghanistan = 4,
    Albania = 8,
    Algeria = 12,
    Andorra = 20,
    Angola = 24,
    AntiguaAndBarbuda = 28,
    Azerbaijan = 31,
    Argentina = 32,
    Australia = 36,
    Austria = 40,
    Bahamas = 44,
    Bahrain = 48,
    Bangladesh = 50,
    Armenia = 51,
    Barbados = 52,
    Belgium = 56,
    Bhutan = 64,
    Bolivia = 68,
    BosniaAndHerzegovina = 70,
    Botswana = 72,
    Brazil = 76,
    Belize = 84,
    SolomonIslands = 90,
    BruneiDarussalam = 96,
    Bulgaria = 100,
    /// also Burma == 104
    Myanmar = 104,
    Burundi = 108,
    Belarus = 112,
    Cambodia = 116,
    Cameroon = 120,
    Canada = 124,
    CapeVerde = 132,
    CentralAfricanRepublic = 140,
    SriLanka = 144,
    Chad = 148,
    Chile = 152,
    China = 156,
    TaiwanProvinceOfChina = 158,
    Colombia = 170,
    Comoros = 174,
    Congo = 178,
    /// also CongoTheDemocraticRepublicOfThe = 180
    Zaire = 180,
    CostaRica = 188,
    Croatia = 191,
    Cuba = 192,
    Cyprus = 196,
    CzechRepublic = 203,
    Benin = 204,
    Denmark = 208,
    Dominica = 212,
    DominicanRepublic = 214,
    Ecuador = 218,
    ElSalvador = 222,
    EquatorialGuinea = 226,
    Ethiopia = 231,
    Eritrea = 232,
    Estonia = 233,
    Fiji = 242,
    Finland = 246,
    France = 250,
    FrenchPolynesia = 258,
    Djibouti = 262,
    Gabon = 266,
    Georgia = 268,
    Gambia = 270,
    Germany = 276,
    Ghana = 288,
    Kiribati = 296,
    Greece = 300,
    Grenada = 308,
    Guatemala = 320,
    Guinea = 324,
    Guyana = 328,
    Haiti = 332,
    HolySeeVaticanCityState = 336,
    Honduras = 340,
    HongKong = 344,
    Hungary = 348,
    Iceland = 352,
    India = 356,
    Indonesia = 360,
    IranIslamicRepublicOf = 364,
    Iraq = 368,
    Ireland = 372,
    Israel = 376,
    Italy = 380,
    CoteDIvoire = 384,
    Jamaica = 388,
    Japan = 392,
    Kazakhstan = 398,
    Jordan = 400,
    Kenya = 404,
    KoreaDemocraticPeoplesRepublicOf = 408,
    KoreaRepublicOf = 410,
    Kuwait = 414,
    Kyrgyzstan = 417,
    LaoPeoplesDemocraticRepublic = 418,
    Lebanon = 422,
    Lesotho = 426,
    Latvia = 428,
    Liberia = 430,
    LibyanArabJamahiriya = 434,
    Liechtenstein = 438,
    Lithuania = 440,
    Luxembourg = 442,
    Macao = 446,
    Madagascar = 450,
    Malawi = 454,
    Malaysia = 458,
    Maldives = 462,
    Mali = 466,
    Malta = 470,
    Mauritania = 478,
    Mauritius = 480,
    Mexico = 484,
    Monaco = 492,
    Mongolia = 496,
    MoldovaRepublicOf = 498,
    Montenegro = 499,
    Morocco = 504,
    Mozambique = 508,
    Oman = 512,
    Namibia = 516,
    Nauru = 520,
    Nepal = 524,
    Netherlands = 528,
    NewCaledonia = 540,
    Vanuatu = 548,
    NewZealand = 554,
    Nicaragua = 558,
    Niger = 562,
    Nigeria = 566,
    Norway = 578,
    MicronesiaFederatedStatesOf = 583,
    MarshallIslands = 584,
    Palau = 585,
    Pakistan = 586,
    Panama = 591,
    PapuaNewGuinea = 598,
    Paraguay = 600,
    Peru = 604,
    Philippines = 608,
    Poland = 616,
    Portugal = 620,
    GuineaBissau = 624,
    TimorLeste = 626,
    PuertoRico = 630,
    Qatar = 634,
    Romania = 642,
    RussianFederation = 643,
    Rwanda = 646,
    SaintKittsAndNevis = 659,
    SaintLucia = 662,
    SaintVincentAndTheGrenadines = 670,
    SanMarino = 674,
    SaoTomeAndPrincipe = 678,
    SaudiArabia = 682,
    Senegal = 686,
    Serbia = 688,
    Seychelles = 690,
    SierraLeone = 694,
    Singapore = 702,
    Slovakia = 703,
    Vietnam = 704,
    Slovenia = 705,
    Somalia = 706,
    SouthAfrica = 710,
    Zimbabwe = 716,
    Spain = 724,
    SouthSudan = 728,
    Sudan = 729,
    WesternSahara = 732,
    Suriname = 740,
    Swaziland = 748,
    Sweden = 752,
    Switzerland = 756,
    SyrianArabRepublic = 760,
    Tajikistan = 762,
    Thailand = 764,
    Togo = 768,
    Tonga = 776,
    TrinidadAndTobago = 780,
    UnitedArabEmirates = 784,
    Tunisia = 788,
    Turkey = 792,
    Turkmenistan = 795,
    Tuvalu = 798,
    Uganda = 800,
    Ukraine = 804,
    MacedoniaTheFormerYugoslavRepublicOf = 807,
    Egypt = 818,
    UnitedKingdom = 826,
    TanzaniaUnitedRepublicOf = 834,
    UnitedStates = 840,
    BurkinaFaso = 854,
    Uruguay = 858,
    Uzbekistan = 860,
    Venezuela = 862,
    Samoa = 882,
    Yemen = 887,
    Zambia = 894,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, Deserialize, TryFromPrimitive)]
pub enum RegionCode {
    Africa = 2100,
    America = 2101,
    AmericanContinent = 2102,
    Antilles = 2103,
    ApecCountries = 2104,
    AseanCountries = 2105,
    Asia = 2106,
    Australasia = 2107,
    Balkans = 2108,
    BalticStates = 2109,
    Benelux = 2110,
    BritishIsles = 2111,
    BritishWestIndies = 2112,
    CentralAmerica = 2113,
    Commonwealth = 2114,
    CommonwealthAfricanTerritories = 2115,
    CommonwealthAsianTerritories = 2116,
    CommonwealthAustralasianTerritories = 2117,
    CommonwealthOfIndependentStates = 2118,
    EasternEurope = 2119,
    Europe = 2120,
    EuropeanEconomicArea = 2121,
    EuropeanContinent = 2122,
    /// also EuropeanEconomicCommunity  = 2123
    EuropeanUnion = 2123,
    GsaCountries = 2124,
    MiddleEast = 2125,
    NaftaCountries = 2126,
    NordicCountries = 2127,
    NorthAfrica = 2128,
    NorthAmerica = 2129,
    Oceania = 2130,
    Scandinavia = 2131,
    SouthAmerica = 2132,
    SouthEastAsia = 2133,
    WestIndies = 2134,
    World = 2136,
    SacemTerritories = 3017,
}
