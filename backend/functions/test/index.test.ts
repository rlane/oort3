process.env.FIRESTORE_EMULATOR_HOST = "localhost:8080";

import * as admin from "firebase-admin";
import { upload } from "../lib/index.js";
import sinon = require("sinon");
import { assert } from "chai";

const testEnv = require("firebase-functions-test")();
admin.initializeApp();
const db = admin.firestore();

const mockResponse = () => {
  const res: any = {};
  res.status = sinon.stub().returns(res);
  res.json = sinon.stub().returns(res);
  res.send = sinon.stub().returns(res);
  res.set = sinon.stub().returns(res);
  res.setHeader = sinon.stub().returns(res);
  return res;
};

suite("Oort backend", function () {
  setup(async function () {
    this.timeout(10000);
    await db.recursiveDelete(db.collection("/telemetry"), db.bulkWriter());
  });

  teardown(function () {
    testEnv.cleanup();
  });

  suite("upload", function () {
    test("create-document", async function () {
      let response = mockResponse();
      let request: any = {
        method: "POST",
        body: {
          build: "build",
          code: "code",
          code_size: 42,
          scenario_name: "scenario_name",
          ticks: 100,
          type: "type",
          userid: "testuser",
        },
      };
      await upload(request, response);
      assert(response.send.calledWith("Success"));
      let q = await db.collection("telemetry").get();
      assert.lengthOf(q.docs, 1);
      assert.include(q.docs[0].data(), request.body);
    });
  });
});
