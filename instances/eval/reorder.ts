import pl from "npm:nodejs-polars";

const path = Deno.args[0];

pl.readCSV(path).select("n", "ms", "makespan").writeCSV(path);
