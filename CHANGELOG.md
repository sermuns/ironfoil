# Changelog

## [0.2.1](https://github.com/sermuns/ironfoil/compare/v0.2.0..0.2.1) - 2026-03-26

### 🚀 Features

- add program icon and stop console on Windows by sermuns in [eccd7b5](https://github.com/sermuns/ironfoil/commit/eccd7b564d18f15c5e0558c2e3d91a59aba9b598)

### ⚙️ Miscellaneous Tasks

- release v0.2.1 by sermuns in [c6fb257](https://github.com/sermuns/ironfoil/commit/c6fb2572b54c456775e9b9900acb99a28aa08343)
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
