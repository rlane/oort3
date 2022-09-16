process.env.FIRESTORE_EMULATOR_HOST = "localhost:8084";

import * as admin from "firebase-admin";
import { upload, leaderboard } from "../lib/index.js";
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
          username: "testusername",
        },
      };
      await upload(request, response);
      assert(response.send.calledWith("Success"));
      let q = await db.collection("telemetry").get();
      assert.lengthOf(q.docs, 1);
      assert.include(q.docs[0].data(), request.body);
    });
  });

  suite("leaderboard", function () {
    let i = 0;
    async function entry(
      scenario_name: string,
      userid: string,
      code_size: number,
      ticks: number
    ) {
      i += 1;
      await db.doc("telemetry/test" + i).set({
        type: "FinishScenario",
        scenario_name: scenario_name,
        userid: userid,
        username: userid + "name",
        code_size: code_size,
        ticks: ticks,
        success: ticks != 0,
      });
    }

    test("get", async function () {
      await Promise.all([
        entry("scenario1", "user1", 10, 100),
        entry("scenario1", "user2", 12, 80),
        entry("scenario1", "user2", 11, 90),
        entry("scenario1", "user3", 13, 0),
      ]);

      let response = mockResponse();
      let request: any = {
        method: "GET",
        url: "/leaderboard?scenario_name=scenario1",
      };
      await leaderboard(request, response);
      sinon.assert.calledWith(response.json, {
        lowest_time: [
          { userid: "user2", username: "user2name", time: "1.33" },
          { userid: "user1", username: "user1name", time: "1.67" },
        ],
        lowest_code_size: [
          { userid: "user1", username: "user1name", code_size: 10 },
          { userid: "user2", username: "user2name", code_size: 11 },
        ],
      });
    });

    test("too-many", async function () {
      let promises = [];
      for (let j = 0; j < 20; j++) {
        promises.push(entry("scenario1", "user" + j, 10 + j, 100 + j));
      }
      await Promise.all(promises);

      let response = mockResponse();
      let request: any = {
        method: "GET",
        url: "/leaderboard?scenario_name=scenario1",
      };
      await leaderboard(request, response);
      sinon.assert.calledWith(response.json, {
        lowest_time: [
          { userid: "user0", username: "user0name", time: "1.67" },
          { userid: "user1", username: "user1name", time: "1.68" },
          { userid: "user2", username: "user2name", time: "1.70" },
          { userid: "user3", username: "user3name", time: "1.72" },
          { userid: "user4", username: "user4name", time: "1.73" },
          { userid: "user5", username: "user5name", time: "1.75" },
          { userid: "user6", username: "user6name", time: "1.77" },
          { userid: "user7", username: "user7name", time: "1.78" },
          { userid: "user8", username: "user8name", time: "1.80" },
          { userid: "user9", username: "user9name", time: "1.82" },
        ],
        lowest_code_size: [
          { userid: "user0", username: "user0name", code_size: 10 },
          { userid: "user1", username: "user1name", code_size: 11 },
          { userid: "user2", username: "user2name", code_size: 12 },
          { userid: "user3", username: "user3name", code_size: 13 },
          { userid: "user4", username: "user4name", code_size: 14 },
          { userid: "user5", username: "user5name", code_size: 15 },
          { userid: "user6", username: "user6name", code_size: 16 },
          { userid: "user7", username: "user7name", code_size: 17 },
          { userid: "user8", username: "user8name", code_size: 18 },
          { userid: "user9", username: "user9name", code_size: 19 },
        ],
      });
    });
  });
});
