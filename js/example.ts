import * as prayTimes from "./pray_times.js";

//TODO: autogenerate this from rust, or both rust and js from a single source of truth
interface RustDate {
  year: number;
  month: number;
  day: number;
  lat: number;
  lon: number;
  tz: number;
}

const rustDate: RustDate = JSON.parse(Deno.args[0]);
// const rustDate: RustDate = {
// year: 1911,
// month: 3,
// day: 11,
// lat: 36,
// lon: 10,
// tz: 1
// }
const date = new Date(`${rustDate.year}-${rustDate.month}-${rustDate.day}`);
const cord = [rustDate.lat, rustDate.lon];

const times = prayTimes.PrayTimes().getTimes(date, cord, rustDate.tz, 0, "24h");

console.log(JSON.stringify(times));
