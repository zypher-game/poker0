const { expect } = require("chai");

describe("RiscZero Snark Proof Verify Contract", function () {
  it("------------------------", async function () {
    const [owner] = await ethers.getSigners();

    const controlID = await ethers.deployContract("ControlID");
    const verifier = await ethers.deployContract("RiscZeroGroth16Verifier", [controlID.CONTROL_ID_0(), controlID.CONTROL_ID_1()]);

    const seal = "0x202dba141bd7f07c6671719f3c7e2c572de9043a25f81ac0dd580410d55bccbc12caaeadf1fab7d15a617a7ef3b4c58395da1b8ae9a924a02151dedc1d25f6ff120d5eb9846fca660eced5ad49254b18913aa0995a27e758a4cc802421a197f811dd3f1cdd9357f9c0196c368d1daa5a53392a8749968bb6ed9bf52c903536931a1fb91a453076c218a73c4e407a9cfb13a6461b32fc6354da3d39abe8e13da51594a26638acd609a9a2c296abdb229c18bf7dd7c95fb88add63e770753e42b8185f0904fc14281fe7e1d14bdfb3e0f25e6f2e2dd6899b1b4dc9fc056e0ceb361d0ae1acdc872e688cc1a9730420ed0966ddc01c88d0a6d17c2799d7e3a73aa2";
    const image_id = "0x4a731fb4059b4be6e9f35892f38a643c016918eeb13e1bd06e34456e601e5251";
    const journal = "0x1c09902f7ab269a5118a9e1fe48fec688d2817d7f657e904e139532308b34cd3";
    const post_digest = "0x3d44e6615b2d7248120da524685071de931ee5b3ba657ee8d9ea0cecc2225abf";

    const res = await verifier.verify(seal, image_id, post_digest, journal);

    expect(res).to.equal(true);
  });
});
