# Changelog

## [0.4.1](https://github.com/sermuns/ironfoil/compare/v0.4.0..0.4.1) - 2026-03-29

### 🐛 Bug Fixes

- **(core)** actually perform 2 to the power of.. ^ means xor... by @sermuns in [13461c4](https://github.com/sermuns/ironfoil/commit/13461c4abca38e82e4f5aa0791a4e23a53b28687)
- **(core)** correct progress report for network install by @sermuns in [3019235](https://github.com/sermuns/ironfoil/commit/3019235d99756896180f6f1185ab43e2ade3fb02)

### 🚜 Refactor

- **(core)** create crate features by @sermuns in [0868f25](https://github.com/sermuns/ironfoil/commit/0868f25a53b0fc6fb7f60068cf3cbbeb2c1ddf75)

### 📚 Documentation

- **(cli)** show more of help output by @sermuns in [b513224](https://github.com/sermuns/ironfoil/commit/b51322461fd7e50819ae2a87f3c14bbfff8cf008)

### ⚙️ Miscellaneous Tasks

- **(core)** remove unused crc32fast by @sermuns in [78d9512](https://github.com/sermuns/ironfoil/commit/78d9512cac0fa590b9b281eb6113e161867283b8)
- **(core)** allow too many lines in `serve_http`... by @sermuns in [c5e517d](https://github.com/sermuns/ironfoil/commit/c5e517d4f798f80fe5a88d50b3eac0ba4da8f216)
- release v0.4.1 by Samuel Åkesson in [712191c](https://github.com/sermuns/ironfoil/commit/712191cd09b24368056fbe676e2462750ea7ed49)
## [v0.4.0](https://github.com/sermuns/ironfoil/compare/v0.3.1..v0.4.0) - 2026-03-29

### 🚀 Features

- **(gui)** progress bars for total and per-file by @sermuns in [9b7ded7](https://github.com/sermuns/ironfoil/commit/9b7ded7b4cf1178f1ec5282728c7781242448a84)
- **(gui)** less repaint churning by only repaininting every 100 ms by @sermuns in [9911408](https://github.com/sermuns/ironfoil/commit/991140819db06bfb2d773afbef7e28a1f2719db0)

### 🐛 Bug Fixes

- **(core)** send Ended event everytime installation ends, even in failure by @sermuns in [118db9c](https://github.com/sermuns/ironfoil/commit/118db9ccbd67d833cbb501b3a769a743193c9f23)
- **(core)** better error message for timeout by @sermuns in [e8c8204](https://github.com/sermuns/ironfoil/commit/e8c8204ffa26ca81396bf791693a0279fe1bd175)
- **(core)** more accurate errors by @sermuns in [da024b2](https://github.com/sermuns/ironfoil/commit/da024b2071fb34ca0b2f75aff39ca714774837ab)

### 🚜 Refactor

- **(core)** combine error handling in http server by @sermuns in [c1ae25c](https://github.com/sermuns/ironfoil/commit/c1ae25c169e559c3e918a458fa93255adfea4489)
- remove Arc nonsense where it's not needed by @sermuns in [a83e99d](https://github.com/sermuns/ironfoil/commit/a83e99d9a1ac12ee69b3f51a8dcc650974a63917)
- move UsbProtocol into core, use it instead of for_sphaira bool by @sermuns in [efec4e6](https://github.com/sermuns/ironfoil/commit/efec4e67da3183b1e0c93b75ef30781a2cff35db)
- create a dedicated InstallProgress enum. less CPU churning on CLI, still need to fix GUI.. by @sermuns in [3a5930d](https://github.com/sermuns/ironfoil/commit/3a5930d9d40e4a1eae30ad50321da747ca296909)
- use high-level Read/Write syntax for USB transfers. fix progress reporting. by @sermuns in [f928f09](https://github.com/sermuns/ironfoil/commit/f928f09a27f2de2949e0b3c109a56d8dcd66e0cf)
- make tinfoil install default by @sermuns in [7d02080](https://github.com/sermuns/ironfoil/commit/7d02080ad1fd6a9b78124dff35f45037abbfb53a)

### 📚 Documentation

- **(cli)** update demo gif by @sermuns in [552a0c1](https://github.com/sermuns/ironfoil/commit/552a0c1824958e8a3fa25521676e90da096b977f)
- **(gui)** update screenshots by @sermuns in [1ecf844](https://github.com/sermuns/ironfoil/commit/1ecf844e88ee44a70b66eaf0e1b097b368a14ce7)
- **(gui)** add thin variant of screenshot1 by @sermuns in [38f6bf2](https://github.com/sermuns/ironfoil/commit/38f6bf260d4746ef1c86bd398262ad686a677fa9)

### ⚙️ Miscellaneous Tasks

- **(gui)** minor error reword by @sermuns in [b4452e8](https://github.com/sermuns/ironfoil/commit/b4452e8cf8bad06097f5689870d1286084ab4b76)
- satisfy/configure clippy by @sermuns in [0fd126f](https://github.com/sermuns/ironfoil/commit/0fd126f5a7a4d567f15441649bd56974c130304a)
- release v0.4.0 by @sermuns in [9e15353](https://github.com/sermuns/ironfoil/commit/9e15353fe11cc1c508e7035969c16da07dab5359)
## [v0.3.1](https://github.com/sermuns/ironfoil/compare/v0.3.0..v0.3.1) - 2026-03-27

### 🚀 Features

- **(gui)** add linux desktop entry by @sermuns in [b99bafc](https://github.com/sermuns/ironfoil/commit/b99bafc4ec25e75bc565160b99eecdaf3bc7f55f)

### 🐛 Bug Fixes

- **(ci)** fix caching by @sermuns in [9054979](https://github.com/sermuns/ironfoil/commit/90549790783754341ff42b7dec911c25e0e6f429)

### 🚜 Refactor

- **(cli)** rename TransferType->InstallType by @sermuns in [a4d3c9e](https://github.com/sermuns/ironfoil/commit/a4d3c9e97ab48f8fce86d36a4d0eadb8ff3ce95e)
- **(cli)** deduplicate usb/network install code by @sermuns in [742c090](https://github.com/sermuns/ironfoil/commit/742c0902a49663704eb779e57a946952b730bd98)
- **(core)** break usb modules out into files by @sermuns in [ce5f5ec](https://github.com/sermuns/ironfoil/commit/ce5f5ecaebc9530f441144cdf267930a382fb5a1)
- **(core)** use shared ref instead of owned Arc by @sermuns in [87b70c8](https://github.com/sermuns/ironfoil/commit/87b70c8eb12fa9736fba8da37d78d9c061821a09)
- **(core)** satisfy some clippy lints by @sermuns in [9ae9b60](https://github.com/sermuns/ironfoil/commit/9ae9b60156b7a0ff60a804d4c9158be69ea7ac0a)
- **(gui)** move ugly code into impl StagedFiles by @sermuns in [fbf5c9d](https://github.com/sermuns/ironfoil/commit/fbf5c9ddf7c2a681c7f8444be351d8d0dc1dfb9c)
- **(gui)** satisfy lints by @sermuns in [e4989a8](https://github.com/sermuns/ironfoil/commit/e4989a86af10bf6257ffeed9705ea2c78f3ed3fc)
- consolidate progress events as an enum. fix bug where checking target ip during usb install.. by @sermuns in [6f078e5](https://github.com/sermuns/ironfoil/commit/6f078e548f9fec3ce6239455eac80498eb04a9ad)

### ⚙️ Miscellaneous Tasks

- **(cli)** color-eyre, only show source on debug by @sermuns in [e3ac7f3](https://github.com/sermuns/ironfoil/commit/e3ac7f35cea7b88b01927596a4cfe0e570106007)
- **(core)** fix needless_borrow by @sermuns in [9c7334e](https://github.com/sermuns/ironfoil/commit/9c7334e7373587f738cc743dfb4f23e822866c2d)
- **(gui)** remove unused imports by @sermuns in [f4fece0](https://github.com/sermuns/ironfoil/commit/f4fece04776dd9a889910f882df54c6a93057b8e)
- **(gui)** only enable eframe wayland,x11 on linux by @sermuns in [9cc4f47](https://github.com/sermuns/ironfoil/commit/9cc4f4780a653240f81907ef751b2d16f5e7acb1)
- remove incorrect comment by @sermuns in [605d510](https://github.com/sermuns/ironfoil/commit/605d5104b5c48f59df3aa376b331be0524b99322)
- add cargo fmt precommit by @sermuns in [360e2df](https://github.com/sermuns/ironfoil/commit/360e2df4222d2f017c9cec87f2790e2502da4e9e)
- add pedantic clippy linting by @sermuns in [4aa637f](https://github.com/sermuns/ironfoil/commit/4aa637f676478e396f247abf7648f691395ee6b6)
- allow override version in release just recipe by @sermuns in [885e160](https://github.com/sermuns/ironfoil/commit/885e160cc20becdf33895a5bcc84a0fdecee9ec1)
- release v0.3.1 by @sermuns in [252d599](https://github.com/sermuns/ironfoil/commit/252d5992ebdf38ed5c9c08259c69a97c9499e69e)
## [v0.3.0](https://github.com/sermuns/ironfoil/compare/v0.2.1..v0.3.0) - 2026-03-26

### 🚀 Features

- **(gui)** add network install by @sermuns in [2d91ac5](https://github.com/sermuns/ironfoil/commit/2d91ac52fdd3ef98df03da1bcae5cd99052ff14c)
- **(gui)** add rcm payload injection by @sermuns in [7907931](https://github.com/sermuns/ironfoil/commit/79079313c78440f2b3294845bb576655dce15ab3)

### 🚜 Refactor

- **(gui)** create tabs module by @sermuns in [a161eb0](https://github.com/sermuns/ironfoil/commit/a161eb0c34bd0c1024b1699c0f96424663e29d47)
- **(gui)** create module per tab by @sermuns in [ca025d0](https://github.com/sermuns/ironfoil/commit/ca025d0149b28853ce7ef12bee65e80629744646)

### 🎨 Styling

- **(gui)** remove top heading and restyle home tab by @sermuns in [e9ea4cc](https://github.com/sermuns/ironfoil/commit/e9ea4ccb9505ab8eb81c3b9ba830357b578d059a)
- **(gui)** rcm tab ricing by @sermuns in [62483af](https://github.com/sermuns/ironfoil/commit/62483afec78f2dd681e82acb27baf31f737b9605)

### ⚙️ Miscellaneous Tasks

- release v0.3.0 by @sermuns in [49c31ad](https://github.com/sermuns/ironfoil/commit/49c31adb80d373b857b541d18785ec7516e15770)
## [v0.2.1](https://github.com/sermuns/ironfoil/compare/v0.2.0..v0.2.1) - 2026-03-26

### 🚀 Features

- add program icon and stop console on Windows by @sermuns in [eccd7b5](https://github.com/sermuns/ironfoil/commit/eccd7b564d18f15c5e0558c2e3d91a59aba9b598)

### ⚙️ Miscellaneous Tasks

- release v0.2.1 by @sermuns in [1550fcb](https://github.com/sermuns/ironfoil/commit/1550fcb90509ff3382215b95d9837d281b42ee87)
## [v0.2.0](https://github.com/sermuns/ironfoil/compare/v0.1.6..v0.2.0) - 2026-03-26

### 🚀 Features

- **(core)** add sphaira usb install support by @sermuns in [54c3a49](https://github.com/sermuns/ironfoil/commit/54c3a49611b84f27feaa4a7550a462d3b9e3f88b)
- **(gui)** begin creating GUI by @sermuns in [4e7fe5c](https://github.com/sermuns/ironfoil/commit/4e7fe5c7db6c3debb265ef3da1a7d83a4df46d80)
- **(gui)** style add icons, semi working transfer by @sermuns in [0d0822c](https://github.com/sermuns/ironfoil/commit/0d0822c2c9e2fb5f9f8938044a081ef99952ec87)
- **(gui)** add sphaira support by @sermuns in [9de9061](https://github.com/sermuns/ironfoil/commit/9de9061cf5715087a56084db1ea9dfc3bb303c37)
- better Dockerfile, using cargo-chef by @sermuns in [9fa1c38](https://github.com/sermuns/ironfoil/commit/9fa1c38892cfd19b397e3181c5614cea5352c942)
- better error on permission denied by @sermuns in [6a34b90](https://github.com/sermuns/ironfoil/commit/6a34b904c113823c29ebf3ddd474587bb555d69d)
- remove indicatif from core, use mpsc::Channel to synchronize progress by @sermuns in [7e61d74](https://github.com/sermuns/ironfoil/commit/7e61d74f935c88b310bf1aaba73976033eef3669)
- add cancelling and stop using suggestion() by @sermuns in [a44854b](https://github.com/sermuns/ironfoil/commit/a44854b9fdc59a987a7a81b8554a9e5234ae8f57)
- fully working GUI, refactor a lot of code by @sermuns in [8bad5c7](https://github.com/sermuns/ironfoil/commit/8bad5c7eaf541f416a7aa67ae4aa290ea71ff956)
- simplify, reword GUI and error by @sermuns in [d474ab3](https://github.com/sermuns/ironfoil/commit/d474ab39ef4fed08ab1caf9778e3db02b21e70e3)
- add distributioning with `cargo-dist` by @sermuns in [e365d56](https://github.com/sermuns/ironfoil/commit/e365d564a4e3244e51eb15dac243e71a01df3b39)

### 🐛 Bug Fixes

- use installer-generic language in error by @sermuns in [ef1eb8f](https://github.com/sermuns/ironfoil/commit/ef1eb8fbe9b84f24bce273f69acff617456064c6)
- align center by @sermuns in [c986fc8](https://github.com/sermuns/ironfoil/commit/c986fc8ff9d0e609f4092ac5949147de543875a3)
- hopefully make windows correctly parse release version? by @sermuns in [f93bdc0](https://github.com/sermuns/ironfoil/commit/f93bdc0708377321c032dd8531c197a644439ad9)
- symlink media for crates.io.. by @sermuns in [0c69754](https://github.com/sermuns/ironfoil/commit/0c69754e48acf21ac1cf21f3f9f4e5b7d12adcd2)
- enable file_glob by @sermuns in [462958d](https://github.com/sermuns/ironfoil/commit/462958d2d1b01fc1445bdd3b3b866debd3d08496)
- only build & publish CLI by @sermuns in [460f80e](https://github.com/sermuns/ironfoil/commit/460f80e2c7abae9b82faf459b81963e6ce7c3bc8)
- force bash in release notes by @sermuns in [0a39755](https://github.com/sermuns/ironfoil/commit/0a39755829e05b63c4063ef20d04b94bf3c0b55e)
- publish ironfoil-core before cli by @sermuns in [3ee98d8](https://github.com/sermuns/ironfoil/commit/3ee98d8a3cddea41e91c17b8a0b3a87e31804238)

### 🚜 Refactor

- export `GAME_BACKUP_EXTENSIONS` by @sermuns in [782caa1](https://github.com/sermuns/ironfoil/commit/782caa1d20c16032959352b20c05ce8336e87c5b)

### 📚 Documentation

- update README by @sermuns in [d3a4d07](https://github.com/sermuns/ironfoil/commit/d3a4d07ac9099791de81b72f56bf19dc93074da8)
- update demos and begin explaining GUI installation by @sermuns in [c5374ba](https://github.com/sermuns/ironfoil/commit/c5374ba09dbfeefa5d2e927c94eaa4ee696e8c3e)

### ⚙️ Miscellaneous Tasks

- add push recipe by @sermuns in [e647d6c](https://github.com/sermuns/ironfoil/commit/e647d6cd97bda01507e258cdab079768310dbba9)
- add TODO to network by @sermuns in [b66bd08](https://github.com/sermuns/ironfoil/commit/b66bd087e80349c011af17df229f20cbde64b6b9)
- don't optimize debug build by @sermuns in [e53b8d2](https://github.com/sermuns/ironfoil/commit/e53b8d27202ceaba27316810bcf4dc9f7d955636)
- Update README.md (#18) by @binarymelon in [b18fac5](https://github.com/sermuns/ironfoil/commit/b18fac5b8fa37a1b84f95758b26811f6607582f1)
- update README and description for more general title installers by @sermuns in [bf8d530](https://github.com/sermuns/ironfoil/commit/bf8d5304e0de7f7fc4e9347c24d1223843f148c8)
- make features bump minor by @sermuns in [b45f10e](https://github.com/sermuns/ironfoil/commit/b45f10ef0417ae78acf64137182466cb6423d272)
- allow dirty ci by @sermuns in [309bf8b](https://github.com/sermuns/ironfoil/commit/309bf8b7c1b4bf69b6c0714097e9319eeb386951)
- don't use cargo-dist for CLI by @sermuns in [1ec963b](https://github.com/sermuns/ironfoil/commit/1ec963bb91f7dcd731e602ca67a67d1ee1d2081f)
- skip pre-building.. by @sermuns in [bd923e8](https://github.com/sermuns/ironfoil/commit/bd923e835d242dcff4b4008ec684adae05c6f5a8)
- simplify dist by @sermuns in [8507648](https://github.com/sermuns/ironfoil/commit/8507648c49d71f4ed1b52ae13d15b0225b53f86e)
- name cli artifacts without version, as gui does by @sermuns in [f96f199](https://github.com/sermuns/ironfoil/commit/f96f199ae0f4f8e43316f3d4c1ec0ed1c2a8a32f)
- add release notes to gui release too, cleanup.. by @sermuns in [a7c0aee](https://github.com/sermuns/ironfoil/commit/a7c0aeeeba257164a12dbdba531443cfee0bc275)
- stop releasing for i686 linux by @sermuns in [abc0cb2](https://github.com/sermuns/ironfoil/commit/abc0cb2713e78637116f1083a838d0a6b66ad6ea)
- add rust-cache by @sermuns in [0945ba8](https://github.com/sermuns/ironfoil/commit/0945ba879977c17a9920cdf4e445ee10225de186)
- release v0.2.0 by @sermuns in [8a3db31](https://github.com/sermuns/ironfoil/commit/8a3db31b0f67a3149249a05e66c5b8b36abecb82)
## [v0.1.6](https://github.com/sermuns/ironfoil/compare/v0.1.5..v0.1.6) - 2026-03-05

### 🚀 Features

- add RCM payload injection by @sermuns in [b5f0ef1](https://github.com/sermuns/ironfoil/commit/b5f0ef1cce28c5ecac7aac47cd6097dbc8d3db8a)

### 🐛 Bug Fixes

- **(ci)** more robust RELEASE_VERSION parsing and release notes handling by @sermuns in [a33f2df](https://github.com/sermuns/ironfoil/commit/a33f2dfb0e3e1e3988e5761a3305d38209edf1ee)

### ⚙️ Miscellaneous Tasks

- release v0.1.6 by @sermuns in [5845411](https://github.com/sermuns/ironfoil/commit/58454118fa967d80f2ee27039a6fa3308d5871dd)
## [v0.1.5](https://github.com/sermuns/ironfoil/compare/v0.1.4..v0.1.5) - 2026-03-05

### 🚀 Features

- smaller binaries by @sermuns in [d2f1756](https://github.com/sermuns/ironfoil/commit/d2f1756091415f131e09c23e64fffb0b0b054153)

### 🚜 Refactor

- split lib into multiple modules by @sermuns in [e9e5ec4](https://github.com/sermuns/ironfoil/commit/e9e5ec4636426b7501b038c5b5386f4f14f23789)

### ⚙️ Miscellaneous Tasks

- release v0.1.5 by @sermuns in [5d8ecbf](https://github.com/sermuns/ironfoil/commit/5d8ecbf20aed04e7943cd36aca6b12a09d2c47e6)
## [v0.1.4](https://github.com/sermuns/ironfoil/compare/v0.1.3..v0.1.4) - 2026-03-04

### 🚀 Features

- add recurse flag by @sermuns in [d649cb5](https://github.com/sermuns/ironfoil/commit/d649cb5fe8049ac04e51dffbe97ad8d60a3b6b1b)

### 📚 Documentation

- update install instructions by @sermuns in [2b65ec2](https://github.com/sermuns/ironfoil/commit/2b65ec266f53817e25e7be387914d499db0b8fc0)

### ⚙️ Miscellaneous Tasks

- simplify release naming by @sermuns in [9b12e01](https://github.com/sermuns/ironfoil/commit/9b12e018c535243f333d937b14e060826df1f9e8)
- add logo, media by @sermuns in [dbf2693](https://github.com/sermuns/ironfoil/commit/dbf269394245242a3f1357034b953bd01a00567c)
- add logo to README by @sermuns in [bdf2305](https://github.com/sermuns/ironfoil/commit/bdf2305a24c515578233dc7ef739f67846e75ff8)
- update logo, hardcode text paths by @sermuns in [d96dcd0](https://github.com/sermuns/ironfoil/commit/d96dcd0c7a2dc5741d0f09eb72f9818004b075bc)
- make banner transparent by @sermuns in [d888462](https://github.com/sermuns/ironfoil/commit/d8884621eec61227fa1aabc2c4b87ce286548365)
- release v0.1.4 by @sermuns in [41692f1](https://github.com/sermuns/ironfoil/commit/41692f1d6fb5c52b67e300891b827e2adf558eb1)
## [v0.1.3](https://github.com/sermuns/ironfoil/compare/v0.1.2..v0.1.3) - 2026-03-04

### ⚙️ Miscellaneous Tasks

- revamp how releasing is done by @sermuns in [b56dae8](https://github.com/sermuns/ironfoil/commit/b56dae8102496dad6068069c732f72e1bf00ae93)
- add Dockerfile, but remove workflow for now by @sermuns in [87da659](https://github.com/sermuns/ironfoil/commit/87da659dee16a30b0be211a6b44b96dde620cc1e)
- use correct token by @sermuns in [4887c2b](https://github.com/sermuns/ironfoil/commit/4887c2b34b29280531544351b2f823f438d77b54)
- add write permission to workflow by @sermuns in [2047fa8](https://github.com/sermuns/ironfoil/commit/2047fa8d4def2f570d7166e1f558aa5961f9e41a)
- release v0.1.3 by @sermuns in [3765e2a](https://github.com/sermuns/ironfoil/commit/3765e2a01566b1ca35e95145fff66664e22ac141)
## [v0.1.2](https://github.com/sermuns/ironfoil/compare/ironfoil-v0.1.1..v0.1.2) - 2026-03-04

### ⚙️ Miscellaneous Tasks

- oops by @sermuns in [445f29c](https://github.com/sermuns/ironfoil/commit/445f29ccddd86ab4b2bfb833440e6a1670131c34)
- adjust tag naming by @sermuns in [bd0555d](https://github.com/sermuns/ironfoil/commit/bd0555dac0ebf218e03015ff1888519ae0231b0d)
- Release by @sermuns in [7291bea](https://github.com/sermuns/ironfoil/commit/7291bea8f3bdfb73356b819ba0905c25de6c0b52)
## [ironfoil-v0.1.1](https://github.com/sermuns/ironfoil/compare/ironfoil-v0.1.0..ironfoil-v0.1.1) - 2026-03-04

### ⚙️ Miscellaneous Tasks

- use correct syntax for selecting target.. by @sermuns in [5648271](https://github.com/sermuns/ironfoil/commit/564827116d342f326e5efd709b201824ec98ca8e)
- do not publish by @sermuns in [2be90ab](https://github.com/sermuns/ironfoil/commit/2be90ab71d963e46ee23cfb45c7db630a02f8495)
- Release by @sermuns in [f68bb00](https://github.com/sermuns/ironfoil/commit/f68bb00308636f51db27abbe8b765ba926af28dd)
## [ironfoil-v0.1.0] - 2026-03-04

### 🚀 Features

- first working implementation! by @sermuns in [eeef3bd](https://github.com/sermuns/ironfoil/commit/eeef3bda6922c1600f4e995d957d5dc606983ed6)
- add progress indicator by @sermuns in [ff53d44](https://github.com/sermuns/ironfoil/commit/ff53d4410754250b1397419b76924a73c7f3f142)
- style the progress indicator by @sermuns in [ef6355c](https://github.com/sermuns/ironfoil/commit/ef6355c12d1f027df83d6ea6956b24467e4a66b6)
- better user-facing error reporting by @sermuns in [8e2f90e](https://github.com/sermuns/ironfoil/commit/8e2f90ea63aa72dd754e9c2a51a5a2848ba19e59)
- allow single NSP transfers by @sermuns in [22bf173](https://github.com/sermuns/ironfoil/commit/22bf173b87931b99cf414f01754318ad52bf495d)
- support XCI format by @sermuns in [b02da39](https://github.com/sermuns/ironfoil/commit/b02da39cfec757916f8166fb7bbbe478fe10b5e0)
- support NSZ format by @sermuns in [65744f7](https://github.com/sermuns/ironfoil/commit/65744f7a9aff62ed38aad75b46d90f5383406f7c)
- **breaking** create subcommands `usb` and `network` and add network transfer by @sermuns in [8303b52](https://github.com/sermuns/ironfoil/commit/8303b526a993007041addc76565e4521168bcc44)

### 🐛 Bug Fixes

- use generic game backup name by @sermuns in [0f7882b](https://github.com/sermuns/ironfoil/commit/0f7882b737dff6913484fa02277e2f9c3a686161)
- spawn thread for HTTP server by @sermuns in [70edf29](https://github.com/sermuns/ironfoil/commit/70edf29fbdfb7be982d48515704bf36d5c4aaa83)
- use cross-platform `.len()` instead of `.size()` by @sermuns in [0f1ac60](https://github.com/sermuns/ironfoil/commit/0f1ac604e6cd066c6da45b9432579a2373285319)

### 💼 Other

- use less features of color-eyre by @sermuns in [3e617dd](https://github.com/sermuns/ironfoil/commit/3e617dd906452ea71a796e32eeb3de6aa81c3235)

### 🚜 Refactor

- minor changes and code comments by @sermuns in [167d3bb](https://github.com/sermuns/ironfoil/commit/167d3bb749e409e99769cee0aed4468be0d6e0d7)
- remove unused import by @sermuns in [84f0d67](https://github.com/sermuns/ironfoil/commit/84f0d67024babfeba8dea6f9354d65a7dac2e098)
- move some code, remove old comments by @sermuns in [b073ad8](https://github.com/sermuns/ironfoil/commit/b073ad85c6512e6785286835340b52292abe8e2a)
- split code into crates `core` and `cli` #5 by @sermuns in [28d99a5](https://github.com/sermuns/ironfoil/commit/28d99a5a1883fb4ae2cd4c841b86f2a84b544681)
- move crates from crates/ into root dir by @sermuns in [be957dd](https://github.com/sermuns/ironfoil/commit/be957ddf2ecb5d07377be362a30c8652995bb4c0)

### 📚 Documentation

- create README by @sermuns in [ae556cd](https://github.com/sermuns/ironfoil/commit/ae556cd6d25be87960a9e4e6cfa5bdb313445306)
- NSP, XCI and NSZ are supported by @sermuns in [564152d](https://github.com/sermuns/ironfoil/commit/564152d8ddb4ccb6c4a524958b3ffcb9601092ca)
- add demo gif, rephrase README by @sermuns in [4259e16](https://github.com/sermuns/ironfoil/commit/4259e16e4cd8539e6cf50eb2dc989116cf9451a8)
- update README by @sermuns in [4bea053](https://github.com/sermuns/ironfoil/commit/4bea053f67106ef029c47e2fa6551372b56b079e)
- update install instruction by @sermuns in [9a35be1](https://github.com/sermuns/ironfoil/commit/9a35be114afc4bef759e8dea4af6fad1f30da124)
- update README by @sermuns in [309e9d5](https://github.com/sermuns/ironfoil/commit/309e9d5d56ba4d84071b6198cb7f283ab6b9fe9b)

### ⚙️ Miscellaneous Tasks

- initial commit by @sermuns in [91b36dc](https://github.com/sermuns/ironfoil/commit/91b36dc1438bd118042f5c10f8085cc55ea66b39)
- add precommits by @sermuns in [561f852](https://github.com/sermuns/ironfoil/commit/561f852a53aed2aeb2b0979ae61c0d9470e8dfae)
- update description and cli meta by @sermuns in [604a60a](https://github.com/sermuns/ironfoil/commit/604a60a225c58f3da831d6869c9109e92d3818de)
- update description by @sermuns in [3601d84](https://github.com/sermuns/ironfoil/commit/3601d843c2aabd03e600f38a08d0bc066ec99f74)
- update README by @sermuns in [6a8c11a](https://github.com/sermuns/ironfoil/commit/6a8c11afe01bfa370c73daebfd008d249533eb80)
- update description in README by @sermuns in [f5ae646](https://github.com/sermuns/ironfoil/commit/f5ae646ba919b40cddb4b3746e78d0e22d18b5b2)
- rephrase error by @sermuns in [17af04a](https://github.com/sermuns/ironfoil/commit/17af04a90e19a0bca23bdd939a1e3e9c4181791d)
- add license by @sermuns in [43ca476](https://github.com/sermuns/ironfoil/commit/43ca476a53b742a271710a8a408b9a920a9a54de)
- remove unused imports by @sermuns in [7de47bb](https://github.com/sermuns/ironfoil/commit/7de47bb1a45bfc6bf29005b4225f8137996a9962)
- rename project from ns-usbloader-rs to ironfoil by @sermuns in [52b5bcc](https://github.com/sermuns/ironfoil/commit/52b5bccfdc22c8ee67c245d481c2d5082bfea28f)
- update GIF by @sermuns in [12c93cc](https://github.com/sermuns/ironfoil/commit/12c93cc1129a02681eaa5c4dd41609a3dffada05)
- add CD and docker workflows by @sermuns in [772c725](https://github.com/sermuns/ironfoil/commit/772c72543eeec05efe29cd7d15759740f7ef06fd)
- fixes to crate metadata for publishing by @sermuns in [7cc31df](https://github.com/sermuns/ironfoil/commit/7cc31df5eaddc204e5b43e5e956da5ae6fc4b49d)
- add release config by @sermuns in [19d6f9c](https://github.com/sermuns/ironfoil/commit/19d6f9c7039dbc1c7aacc93fc818aa18c0ac28d9)
- Release by @sermuns in [05985c5](https://github.com/sermuns/ironfoil/commit/05985c5a2d80d62bce11a9cf10355019354662d3)
