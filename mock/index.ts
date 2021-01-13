const polka = require("polka");
import { json } from "body-parser";
import { v4 } from "uuid";
import axios from "axios";

class RawCloudWatchLog {
  constructor(record: any) {
    this.time = new Date(Date.now()).toISOString();
    this.record = record;
  }
  time: String;
  type: string = "function";
  record: any;
}

class LogsApiConfig {
  destination: {
    protocol: string;
    URI: string;
  };
  types: string[];
  buffering: {
    maxItems: number;
    maxBytes: number;
    timeoutMs: number;
  };
}

class Tracing {
  type: string;
  value: string;
}

class InvokeResponse {
  eventType: string = "INVOKE";
  deadlineMs: number;
  requestId: string;
  invokedFunctionArn: string;
  tracing: Tracing;
}

class ShutdownResponse {
  eventType: string = "SHUTDOWN";
  shutdownReason: string;
  deadlineMs: string;
}

type NextEventResponse = InvokeResponse | ShutdownResponse;

class ExtensionsApiRequest {
  events: string[];
}

let logs: RawCloudWatchLog[] = [];

let logsApiConfig: LogsApiConfig;
let hasConnection: boolean = false;

function delay(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

const mockFunction = async (): Promise<RawCloudWatchLog[]> => {
  await delay(300);
  return [
    new RawCloudWatchLog({ testData: v4(), level: "INFO" }),
    new RawCloudWatchLog({ testData: v4(), level: "INFO" }),
    new RawCloudWatchLog({ testData: v4(), level: "INFO" }),
    new RawCloudWatchLog({ testData: v4(), level: "INFO" }),
    new RawCloudWatchLog({ testData: v4(), level: "INFO" }),
    new RawCloudWatchLog({ testData: v4(), level: "INFO" }),
    new RawCloudWatchLog({ testData: v4(), level: "INFO" })
  ];
};

polka()
  .listen(3000)
  .use(json())
  .put("/2020-08-15/logs", (req, res) => {
    logsApiConfig = req.body as LogsApiConfig;
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end();
  })
  .post("/2020-01-01/extension/register", (req, res) => {
    res.writeHead(200, {
      "Content-Type": "application/json",
      "Lambda-Extension-Identifier": v4()
    });
    res.end();
  })
  .get("/2020-01-01/extension/event/next", async (req, res) => {
    while (hasConnection) {
      await delay(100);
    }
    hasConnection = true;
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(
      JSON.stringify({
        deadlineMs: 1000,
        requestId: v4(),
        tracing: { type: "string", value: v4() },
        eventType: "INVOKE",
        invokedFunctionArn: v4()
      } as InvokeResponse)
    );
    let l = await mockFunction();
    logs = logs.concat(l);
    if (Math.random() > 0.8) {
      await axios.post(
        logsApiConfig.destination.URI,
        JSON.stringify(logs.map(log=> {log.record = "2020-11-18T23:52:30.128Z 6e48723a-1596-4313-a9af-e4da9214d637 INFO "+JSON.stringify(log.record); return log})),
        {
          headers: { "Content-Type": "application/json" }
        }
      );
      logs = [];
    }
    hasConnection = false;
  });
