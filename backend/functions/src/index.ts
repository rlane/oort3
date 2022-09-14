import * as functions from "firebase-functions";
import { Firestore } from "@google-cloud/firestore";
import url = require("url");

const firestore = new Firestore();

export const upload = functions.https.onRequest(async (request, response) => {
  response.set("Access-Control-Allow-Origin", "*");

  if (request.method === "OPTIONS") {
    response.set("Access-Control-Allow-Methods", "GET");
    response.set("Access-Control-Allow-Headers", "Content-Type");
    response.set("Access-Control-Max-Age", "3600");
    response.status(204).send("");
    return;
  }

  const document = firestore.doc("telemetry/" + Date.now());
  functions.logger.info("Document path: " + document.path);
  functions.logger.info("Request body " + JSON.stringify(request.body));
  await document.set(request.body);
  response.send("Success");
});

export const leaderboard = functions.https.onRequest(
  async (request, response) => {
    response.set("Access-Control-Allow-Origin", "*");

    if (request.method === "OPTIONS") {
      response.set("Access-Control-Allow-Methods", "GET");
      response.set("Access-Control-Allow-Headers", "Content-Type");
      response.set("Access-Control-Max-Age", "3600");
      response.status(204).send("");
      return;
    }

    const parsedUrl = url.parse(request.url, true);

    const scenarioName = parsedUrl.query.scenario_name;
    if (typeof scenarioName != "string") {
      response.status(400).send("Missing scenario_name parameter");
      return;
    }

    const leaderboard = await makeLeaderboard(firestore, scenarioName);
    response.json(leaderboard);
  }
);

interface StringSet {
  [index: string]: boolean;
}

async function makeLeaderboard(firestore: Firestore, scenarioName: string) {
  const collectionReference = firestore.collection("telemetry");
  const tickQuery = await collectionReference
    .where("type", "==", "FinishScenario")
    .where("scenario_name", "==", scenarioName)
    .orderBy("ticks")
    .limit(100)
    .get();
  const codeSizeQuery = await collectionReference
    .where("type", "==", "FinishScenario")
    .where("scenario_name", "==", scenarioName)
    .orderBy("code_size")
    .limit(100)
    .get();
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const response: any = { lowest_time: [], lowest_code_size: [] };
  let seenUsers: StringSet = {};
  for (const doc of tickQuery.docs) {
    const userid = doc.get("userid");
    if (userid === undefined || userid in seenUsers) {
      continue;
    }
    const username = doc.get("username");
    seenUsers[userid] = true;
    response.lowest_time.push({
      userid: userid,
      username: username,
      time: (doc.get("ticks") * (1.0 / 60)).toFixed(2),
    });
    if (response.lowest_time.length >= 10) {
      break;
    }
  }
  seenUsers = {};
  for (const doc of codeSizeQuery.docs) {
    const userid = doc.get("userid");
    if (userid === undefined || userid in seenUsers) {
      continue;
    }
    const username = doc.get("username");
    seenUsers[userid] = true;
    response.lowest_code_size.push({
      userid,
      username: username,
      code_size: doc.get("code_size"),
    });
    if (response.lowest_code_size.length >= 10) {
      break;
    }
  }
  return response;
}
