// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

library VerifierKey {
    function load(uint256 vk, uint256 pi) internal pure {
        assembly {
            // The commitments of the selectors (9).
            mstore(add(vk, 0x0), 0x0882846857285d8b4ad9f94478b178dd49a11965c74d0188ec6feab9c66bb0af)
            mstore(add(vk, 0x20), 0x04ae83dc93af188681bc689b1814ee9dee33c4a581ac0e4545dc55549e7ddd0d)
            mstore(add(vk, 0x40), 0x14fd63aaebea3a6a1b0a83341a85fe12a5ff3f47ffd630ba17f90b266e8a97cd)
            mstore(add(vk, 0x60), 0x07f0aef636226a3cada11327089fa3b4259edbc15f5d030fabcdb5f11c56535f)
            mstore(add(vk, 0x80), 0x0be6545e1357c3202e4bf8b37ed980bdf612da5f5f2ed6fa19ad722ec48957da)
            mstore(add(vk, 0xa0), 0x0d891367f68155a5dddd779e1c3479dd4f392742b60ae70d86658d29f300107c)
            mstore(add(vk, 0xc0), 0x13e5181ecc014771bd567d5bb3ef933e7db2ff359beb0b33f626165a4363b907)
            mstore(add(vk, 0xe0), 0x17cb29ad134a2c78019a41f87646386f819ba7cfff3c9562bb8f48989373bfc1)
            mstore(add(vk, 0x100), 0x252373868019a869290c893ff21fda360785a6d243e3a91f73c63f5dec14ba32)
            mstore(add(vk, 0x120), 0x0ddc14f165795af331458273d56b01e684f0850e94a86ef0669c9c78bcb0dd80)
            mstore(add(vk, 0x140), 0x00579db99a0f586436efb387ae6f077597608f289dcd8251316a4446a2ab221a)
            mstore(add(vk, 0x160), 0x0436e1529e492418751498a06db537b9ebdde1c8b1f30c5feecea75cb41de431)
            mstore(add(vk, 0x180), 0x0de023c05b98d35a47fadfa988c69b79126928805e6b04de00e0bf3faa879c83)
            mstore(add(vk, 0x1a0), 0x1ec55abc8a4b3eab9a5abd50a5cc7c46a64d39cd6da78aae995db874815f7cd4)
            mstore(add(vk, 0x1c0), 0x27573180c9556a67c2d22d95c09062f125a235abc136156ded124729314876c3)
            mstore(add(vk, 0x1e0), 0x07ea4d67980a6f9089e0db0bc88984d41a5499c79500b1d97923673689746728)
            mstore(add(vk, 0x200), 0x2e80481fe15527881533a6ce794903507679fb946043185c56906739e529d78e)
            mstore(add(vk, 0x220), 0x15f9db4b58d5d5411487b327e7749841bf04f3445638e88683373aa80a62238f)

            // The commitments of perm1, perm2, ..., perm_{n_wires_per_gate}.
            mstore(add(vk, 0x240), 0x0662b6d61911239764d33796e9810840c7d2271b096273f5db88bb4307b26a37)
            mstore(add(vk, 0x260), 0x13e03eb49e7ab9dccd15fda213dadd66ede3773f28f6f60687dfffbb8fcd7faf)
            mstore(add(vk, 0x280), 0x1788f61ddb95f6687819f63357f4e87220a037f8f3199e2bc6453bfc3f57f7c2)
            mstore(add(vk, 0x2a0), 0x1d7c9cdf0971ec2a109b5139e4ff9dc21e559a030ddfcb0873e008d31884df2b)
            mstore(add(vk, 0x2c0), 0x26238da66d50bf2553c802dbdda9508087a2f539e65c3f404ec2f9f72f5fb169)
            mstore(add(vk, 0x2e0), 0x21145a388783bf52f652e5f1943b47cdca79ef0dd607d6be32b5d9656c1858a2)
            mstore(add(vk, 0x300), 0x19338541397885cdabade6d2b6535dd9d2c6a4039a30c012c853e37e35c1ba8d)
            mstore(add(vk, 0x320), 0x14aeb46ef348241542759434539438f36197a5ab1bc7674488a93fbcf16ca59b)
            mstore(add(vk, 0x340), 0x11287ae85399fce0e7e913610e53316934e80ec33a9f4bc493503af147ed2c40)
            mstore(add(vk, 0x360), 0x172c6bb38576252dce6fb4d0484ddd85e1d239569ba6af17c745a6c8bdef0a32)

            // The commitment of the boolean selector.
            mstore(add(vk, 0x380), 0x0df526fe3e10b706ac746f7389b92ee4465a067eaaf0cf2e86eec0d1d4b1c612)
            mstore(add(vk, 0x3a0), 0x21e8c74eb98ac856006c9b8367df7c14d76e530c099e75790b8f53f64552eae3)

            // The commitments of the preprocessed round key selectors.
            mstore(add(vk, 0x3c0), 0x2615c3afb18596c9f64887fe1be649e624d29195f2dabbfb3421a3bb6409b5f7)
            mstore(add(vk, 0x3e0), 0x202f3a3392d6f6d7dde7e0052cb15a255a9d9178c660a448006fa7afcb11fd23)
            mstore(add(vk, 0x400), 0x26abb2042501cae9c737163d97c063108bd5f711733f9eed65713a7a314c9315)
            mstore(add(vk, 0x420), 0x27da8eb04c5a73aa8d7be2b0276e78d000f2f0aa3210aff423b0bbb04e1cc8b0)
            mstore(add(vk, 0x440), 0x2fc03a614b56ee33a61a75ac0bafc8a592834dccc779696d2f46b0992b06b3c9)
            mstore(add(vk, 0x460), 0x2c3ee977456bfbef99205e0aba2d7ce8c4e0f7275f2b5fc7b1c2fe78506b995f)
            mstore(add(vk, 0x480), 0x1f49b702add3f3f30b9a53a26da7cdc80bc475a95399719637faa37269f98212)
            mstore(add(vk, 0x4a0), 0x095a90793a7b54fdecae5008ebad91a9713b77544baf9e97ab1f4e98e30171ef)

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

            mstore(add(pi, 0x0), 356)
        }
    }
}
