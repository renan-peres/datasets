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
async function generateMonetaryBaseCSV() {
  try {
    // Fetch all series
    const [monetaryBase, m1Stock, m2Stock, currencyCirc] = await Promise.all([
      fetchFredData('BOGMBASE'),
      fetchFredData('M1SL'),
      fetchFredData('M2SL'),
      fetchFredData('MBCURRCIR')
    ]);

    // Create a map of dates to combine all series
    const dataByDate = new Map();

    // Process each series and add to the map
    monetaryBase.forEach(obs => {
      if (!dataByDate.has(obs.date)) {
        dataByDate.set(obs.date, {
          date: obs.date,
          monetary_base: parseFloat(obs.value),
          m1_stock: null,
          m2_stock: null,
          currency_circulation: null
        });
      } else {
        dataByDate.get(obs.date).monetary_base = parseFloat(obs.value);
      }
    });

    m1Stock.forEach(obs => {
      if (!dataByDate.has(obs.date)) {
        dataByDate.set(obs.date, {
          date: obs.date,
          monetary_base: null,
          m1_stock: parseFloat(obs.value),
          m2_stock: null,
          currency_circulation: null
        });
      } else {
        dataByDate.get(obs.date).m1_stock = parseFloat(obs.value);
      }
    });

    m2Stock.forEach(obs => {
      if (!dataByDate.has(obs.date)) {
        dataByDate.set(obs.date, {
          date: obs.date,
          monetary_base: null,
          m1_stock: null,
          m2_stock: parseFloat(obs.value),
          currency_circulation: null
        });
      } else {
        dataByDate.get(obs.date).m2_stock = parseFloat(obs.value);
      }
    });

    currencyCirc.forEach(obs => {
      if (!dataByDate.has(obs.date)) {
        dataByDate.set(obs.date, {
          date: obs.date,
          monetary_base: null,
          m1_stock: null,
          m2_stock: null,
          currency_circulation: parseFloat(obs.value)
        });
      } else {
        dataByDate.get(obs.date).currency_circulation = parseFloat(obs.value);
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

generateMonetaryBaseCSV();