// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

library VerifierKey {
    function load(uint256 vk, uint256 pi) internal pure {
        assembly {
            // The commitments of the selectors (9).
            mstore(add(vk, 0x0), 0x1f187bdb84a97ff50bf4ca5866fecd965c63464ce9514e1a122381d65e5916c4)
            mstore(add(vk, 0x20), 0x2a04ed516e88a9350e8498609ce3ba69b91e238965bd8b0fd56f1bf0ec5c008d)
            mstore(add(vk, 0x40), 0x03b8cac58bed9e2880d2908ea3114bffcbd3f61f9adb5ccf72814989cacbe23c)
            mstore(add(vk, 0x60), 0x1dd237c2ce6674e51e5226c53f724cc1b1a95b172b0ec5f3998b7d8a53f65927)
            mstore(add(vk, 0x80), 0x0a9739b67d8ad878f9b5d9be368310f73ce2b52d56f1a517390343f23640b946)
            mstore(add(vk, 0xa0), 0x1a75296b2d85ac3d41850b6e21632968911e89942cf5fff9902cb34f0ecb106e)
            mstore(add(vk, 0xc0), 0x165c3df1552787f2b141439d5418e3d7b1750c4d961038594bf72023897259d0)
            mstore(add(vk, 0xe0), 0x1c5b6dd15f3b0330a8d0b1f421ed7ede04c043d33e8c5afe689217a13cc3a5b8)
            mstore(add(vk, 0x100), 0x0a8ac5e17a210fd523280fc5bbad28da6592b3c7387b9f5dff804d7e2cb4e001)
            mstore(add(vk, 0x120), 0x16bb6ae430b2dccb413bdc4d6d18bfc94fdf7000c985f40ec11af672c882ea1a)
            mstore(add(vk, 0x140), 0x0bce6b53e9b301168e2622f53e95f34d98703d569ad7936a97f37b0258080438)
            mstore(add(vk, 0x160), 0x2274f19c3082492db16c3d7d7415e7cda93c467252afb94b90885cac48351103)
            mstore(add(vk, 0x180), 0x04c5ceee9948249e1da2e1a4c45c81f0959e401cb8efad4e2959da0914d19bb8)
            mstore(add(vk, 0x1a0), 0x200058e2320429bd53d8cae8481029dfd33a7951e58476599fd963d5406c6cab)
            mstore(add(vk, 0x1c0), 0x282928fd503cddd8d6a5c89f7977bcd8c8b6f57fddc0852fc80b4b41642f9fba)
            mstore(add(vk, 0x1e0), 0x2b0dce9f3424cee59b83cd1db5e54ae9e2c3c438d70002b8461d7da57ee9ead0)
            mstore(add(vk, 0x200), 0x093309467d8350a7883d95f7b8f0fbd838568afa1f92496c648eac02769b0ace)
            mstore(add(vk, 0x220), 0x2af27043b539b0bcc587a7b2f08a210c9ad34f35cff0779f94ffce52ffa3292c)

            // The commitments of perm1, perm2, ..., perm_{n_wires_per_gate}.
            mstore(add(vk, 0x240), 0x109c496583f5126624a85e0d13fd0c127fc29f2c6d6ee62cbc2a58b6cb9d3949)
            mstore(add(vk, 0x260), 0x016f11ac5d8620f43c321cd26c79df24827a11df7bf9d5684a52363a09e065d0)
            mstore(add(vk, 0x280), 0x06c2b5ace6e7bf9001047f82ad2645d3c45707690ac366ab3bb6f140cc8d93ca)
            mstore(add(vk, 0x2a0), 0x2e254b2fd1724699bf6fa668670de35638668838a11bd7019e960af78a85adb8)
            mstore(add(vk, 0x2c0), 0x17a4281fe499fd8418a1b64659f4395444e2f8d84243b6c062bcdce778d4ba7f)
            mstore(add(vk, 0x2e0), 0x012840edf4fe4f4d63988dd5e8d3bfe8ebef5aaa4bb57e0c06e9455d26175794)
            mstore(add(vk, 0x300), 0x24b639ffe31246ab2e12b8b91937b7b3c8fc4e2a5f7dfb5389b397d8f842f43a)
            mstore(add(vk, 0x320), 0x1912e14488c6a7f8fdad8a1b24b670e72d0cce87f4fe7c1c298bf7a7f704c99a)
            mstore(add(vk, 0x340), 0x0d772a008990118a8171761965bf8fd222ac107a93e971dff56d0ce95c3b9b76)
            mstore(add(vk, 0x360), 0x1cf3c80953c23cac5542ff293e2d808da5f613aa899bdb72ed39e21a62f20293)

            // The commitment of the boolean selector.
            mstore(add(vk, 0x380), 0x0f203a290d1a854408ea90dd77d5ab1e0d77cc012f65f5d015c9a6f7416ca32d)
            mstore(add(vk, 0x3a0), 0x2045711417bc3382298a8efa0c004254a3b5585baac0ec0953d64d4da886e922)

            // The commitments of the preprocessed round key selectors.
            mstore(add(vk, 0x3c0), 0x090b98b1eca92be91a4bb8b60c428207af49a03bf09fbb1db5acea79a7afbf23)
            mstore(add(vk, 0x3e0), 0x3012e8d6e7690f62f01445ff5f77600860ce54175ee18c679fc4b761eb8fde74)
            mstore(add(vk, 0x400), 0x17b79bf3d37337243c36d8dd3d2620d9942843c24add325a4d9236dfed51cd97)
            mstore(add(vk, 0x420), 0x018cecf4ce01af8a69ee8138221b59f7d792340dc457e5f2d17a9a317699445b)
            mstore(add(vk, 0x440), 0x2aa1ca2d18901e68d86e7fd448db966221af8d0c3f4a8cea7443786c55991243)
            mstore(add(vk, 0x460), 0x1da656dcc41070b578529ea1247e5f079776d2d742a3d47784852e09912ff7d0)
            mstore(add(vk, 0x480), 0x16c608c33dc94e0d6376a0d58b3e61f43c050c9a48097cce25d4e1ae55d0572d)
            mstore(add(vk, 0x4a0), 0x16e1fe4718b0ae63961569facbf4342d66a66d2c2e8f49fafb71cd32531a0ce6)

            // The Anemoi generator.
            mstore(add(vk, 0x4c0), 0x0000000000000000000000000000000000000000000000000000000000000005)

            // The Anemoi generator's inverse.
            mstore(add(vk, 0x4e0), 0x135b52945a13d9aa49b9b57c33cd568ba9ae5ce9ca4a2d06e7f3fbd4c6666667)

            // `n_wires_per_gate` different quadratic non-residue in F_q-{0}.
            mstore(add(vk, 0x500), 0x0000000000000000000000000000000000000000000000000000000000000001)
            mstore(add(vk, 0x520), 0x2f8dd1f1a7583c42c4e12a44e110404c73ca6c94813f85835da4fb7bb1301d4a)
            mstore(add(vk, 0x540), 0x2042a587a90c187b0a087c03e29c968b950b1db26d5c82d666905a6895790c0a)
            mstore(add(vk, 0x560), 0x2db4944e13e6e33cf0ef0734796ff332d73b5fa160dca733bf529e9b758e4960)
            mstore(add(vk, 0x580), 0x1d9e3a4aaf01052d9925138dc6d7d05aa614e311040142458b045d0053d22f46)

            // The domain's group generator with csSize.
            mstore(add(vk, 0x5a0), 0x26125da10a0ed06327508aba06d1e303ac616632dbed349f53422da953337857)

            // The size of constraint system.
            mstore(add(vk, 0x5c0), 1048576)

            mstore(add(pi, 0x0), 328)
        }
    }
}
