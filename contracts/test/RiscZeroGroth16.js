const { expect } = require("chai");

describe("RiscZero Snark Proof Verify Contract", function () {
  it("------------------------", async function () {
    const [owner] = await ethers.getSigners();

    const controlID = await ethers.deployContract("ControlID");
    const verifier = await ethers.deployContract("RiscZeroGroth16Verifier", [controlID.CONTROL_ID_0(), controlID.CONTROL_ID_1()]);

    const seal = "0x125682aa21734b6e3343fd6debcbeef2aea556c39be28ac6c57577eded91acd1027ca020b5930e465512c8ac678634a21a00419f692c26a294a973d6e3bab7220115e711b34896bbbac791c238aff7fecbf0f846fa46bae4f30ca1de0d8d164a09462f03e0f458cbc7599f52d5294cde52c23598e9268290fb30dcb93540cb9f0cd3287ee3ce372fa8888c83938dcb19d870a3b963b4efef21a219c9d75fd2d12a8e2752462b7c5961435b2f83257d1819f5e32997c816b88d0ebdbf15a05fe102d175c0be8fa6f4e04f097a251884c5d4fed9a70a71f509846667d0972d8b7117c591a648f623886f9275280bff054c21316d3dacf3688d0405746bf4620467";
    const image_id = "0xbd385026c9770325324204fe6c7f38f93592efd58fc96460b5eadaf3c061cef4";
    const journal = "0x1c09902f7ab269a5118a9e1fe48fec688d2817d7f657e904e139532308b34cd3";
    const post_digest = "0xfa05273af228be4fe5e285e6d1532e3a63930db68af2b239b0f3e4909d762736";

    const res = await verifier.verify(seal, image_id, post_digest, journal);

    expect(res).to.equal(true);
  });
});
