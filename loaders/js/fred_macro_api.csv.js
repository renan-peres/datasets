import {csvFormat} from "d3-dsv";
import {utcFormat} from "d3-time-format";

const API_KEY = 'e48bedb2c2904cdd62836266863ade1f'; // Replace with your API key
const formatDate = utcFormat("%Y-%m-%d");

// Function to fetch data from FRED
async function fetchFredData(seriesId) {
  const url = `https://api.stlouisfed.org/fred/series/observations?series_id=${seriesId}&api_key=${API_KEY}&file_type=json&sort_order=desc&observation_start=2023-01-01`;
  const response = await fetch(url);
  const data = await response.json();
  return data.observations;
}

// Main function to get all series and combine them
async function generateMacroEconomicCSV() {
  try {
    // Fetch all series
    const [unemployment, gdp, inflation, fedRate, exchangeRate] = await Promise.all([
      fetchFredData('UNRATE'),     // Unemployment Rate
      fetchFredData('GDP'),        // Gross Domestic Product
      fetchFredData('FPCPITOTLZGUSA'), // Inflation Rate
      fetchFredData('DFF'),        // Federal Funds Rate
      fetchFredData('DEXUSEU')     // USD/EUR Exchange Rate
    ]);

    // Create a map of dates to combine all series
    const dataByDate = new Map();

    // Process each series and add to the map
    unemployment.forEach(obs => {
      if (!dataByDate.has(obs.date)) {
        dataByDate.set(obs.date, {
          date: obs.date,
          unemployment_rate: parseFloat(obs.value),
          gdp: null,
          inflation_rate: null,
          fed_funds_rate: null,
          exchange_rate_usd_eur: null
        });
      } else {
        dataByDate.get(obs.date).unemployment_rate = parseFloat(obs.value);
      }
    });

    gdp.forEach(obs => {
      if (!dataByDate.has(obs.date)) {
        dataByDate.set(obs.date, {
          date: obs.date,
          unemployment_rate: null,
          gdp: parseFloat(obs.value),
          inflation_rate: null,
          fed_funds_rate: null,
          exchange_rate_usd_eur: null
        });
      } else {
        dataByDate.get(obs.date).gdp = parseFloat(obs.value);
      }
    });

    inflation.forEach(obs => {
      if (!dataByDate.has(obs.date)) {
        dataByDate.set(obs.date, {
          date: obs.date,
          unemployment_rate: null,
          gdp: null,
          inflation_rate: parseFloat(obs.value),
          fed_funds_rate: null,
          exchange_rate_usd_eur: null
        });
      } else {
        dataByDate.get(obs.date).inflation_rate = parseFloat(obs.value);
      }
    });

    fedRate.forEach(obs => {
      if (!dataByDate.has(obs.date)) {
        dataByDate.set(obs.date, {
          date: obs.date,
          unemployment_rate: null,
          gdp: null,
          inflation_rate: null,
          fed_funds_rate: parseFloat(obs.value),
          exchange_rate_usd_eur: null
        });
      } else {
        dataByDate.get(obs.date).fed_funds_rate = parseFloat(obs.value);
      }
    });

    exchangeRate.forEach(obs => {
      if (!dataByDate.has(obs.date)) {
        dataByDate.set(obs.date, {
          date: obs.date,
          unemployment_rate: null,
          gdp: null,
          inflation_rate: null,
          fed_funds_rate: null,
          exchange_rate_usd_eur: parseFloat(obs.value)
        });
      } else {
        dataByDate.get(obs.date).exchange_rate_usd_eur = parseFloat(obs.value);
      }
    });

    // Convert map to array and sort by date
    const combinedData = Array.from(dataByDate.values())
      .sort((a, b) => b.date.localeCompare(a.date));

    // Output as CSV
    process.stdout.write(csvFormat(combinedData));

  } catch (error) {
    console.error('Error generating CSV:', error);
    process.exit(1);
  }
}

generateMacroEconomicCSV();