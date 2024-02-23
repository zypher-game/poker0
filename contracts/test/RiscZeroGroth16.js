const { expect } = require("chai");

describe("RiscZero Snark Proof Verify Contract", function () {
  it("------------------------", async function () {
    const [owner] = await ethers.getSigners();

    const controlID = await ethers.deployContract("ControlID");
    const verifier = await ethers.deployContract("RiscZeroGroth16Verifier", [controlID.CONTROL_ID_0(), controlID.CONTROL_ID_1()]);

    const seal = "0x1d7198346ef9e678e0c2ed35046797cc751eafd81a8a6f9113df3005473c3d6128559d356c0e6df7a1a91b28b2b9b2949f754ef6dc3ab19a248facc464ce5046072b7856c55f550df25027b99edad62c5b01b41b6d2bb445945acd3a78f82c980ac2302f34511f451579342c20fca70f5d847f750b80c2846f0db8c47a784c34007586341e15df5b30474b594f3fc183e8d476d384c26aea4813d26118ccba5e245bcc78f2405ed2320be3bd7d6081999230335f0f375e25b768f34f806ec0921eea51cd9feec956f194ffb00d1dd44d7e1e00e4f7f42ead59e693f844328f9614dabea433a478a7442a69d4f73fd3a20c4bda1f1bf5f0fa1c36b9fb25622a76";
    const image_id = "0x3b1c3178c0521ff8a8716e4f2f4cb097772a6dfe0e8acad1b466a4a2398ac066";
    const journal = "0x1c09902f7ab269a5118a9e1fe48fec688d2817d7f657e904e139532308b34cd3";
    const post_digest = "0x46e00c941f59b8cffe7d706c73e2d3405883a357db06b17a76e7d1eab283b5c7";

    const res = await verifier.verify(seal, image_id, post_digest, journal);

    expect(res).to.equal(true);
  });
});
