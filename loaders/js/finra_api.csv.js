import { csvFormat } from "d3-dsv";
import fetch from "node-fetch";  // Need to import fetch for Node.js
import btoa from "btoa";  // Need to import btoa for Node.js

// Get token and data in one async function
async function getFinraData() {
  const credentials = {
    clientId: "445db239b8054bbb9542",
    clientSecret: "kbj.uxc5TCQ-vah9cqr"
  };
  
  // Get token
  const authString = btoa(`${credentials.clientId}:${credentials.clientSecret}`);
  const tokenResponse = await fetch("https://ews.fip.finra.org/fip/rest/ews/oauth2/access_token?grant_type=client_credentials", {
    method: 'POST',
    headers: {
      'Authorization': `Basic ${authString}`
    }
  });
  const tokenData = await tokenResponse.json();
  const token = tokenData.access_token;
  
  // Get market data
  const marketResponse = await fetch("https://api.finra.org/data/group/fixedIncomeMarket/name/treasuryMonthlyAggregates?limit=1000", {
    method: 'GET',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Accept': 'application/json',
      'Content-Type': 'application/json'
    }
  });
  const data = await marketResponse.json();
  return data;
}

// Execute and write to stdout
const data = await getFinraData();
process.stdout.write(csvFormat(data));
