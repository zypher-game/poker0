const { expect } = require("chai");

describe("RiscZero Snark Proof Verify Contract", function () {
  it("------------------------", async function () {
    const [owner] = await ethers.getSigners();

    const controlID = await ethers.deployContract("ControlID");
    const verifier = await ethers.deployContract("RiscZeroGroth16Verifier", [controlID.CONTROL_ID_0(), controlID.CONTROL_ID_1()]);

    const seal = "0x2f1e65d77e9e8989b63c99d612743084e7c42a1413caf6674b8545d4b21971aa03abb44ff4a8e3484fe2b634aed0583c25074f4f8dfe40a3f87990bea9a1b9e217a0534f45765285b269b159147ef4a7646a0d3dd9b930fbd59fbd847fb794142cac58b465199f28d93cb0e607c6b5ff04f7e876065340b1b7181206797d850e26a8e1b70b63ed7e6ad010e66a68c329ebf648a5773f64f92f9ee3d0034c803e0202950f405dc3b6e8e54410c0bdaa9a1c7f32c6291fa7c2d264f0039dc4afb401ac1fb609164aaaa4dffecce430e17395f593dc34ef36a99615b349e7a07e541aed282bbbe609e1613630ff19b72b52fe4cb8fda6ec1e9d481c61161785863b";
    const image_id = "0x30f1f5fa1b4dd15f20cf14829ae488184f76c0ca213f36e976aa71d2de1ee88c";
    const journal = "0xe91763a3c52471a3beecccba2001409dbb05b6be83d9a1fbd3458e4a490671af";
    const post_digest = "0xb9b22699c290f082650d30272d2a118bef50226c1416cd5716ba49a401d95b43";

    const res = await verifier.verify(seal, image_id, post_digest, journal);

    expect(res).to.equal(true);
  });
});
