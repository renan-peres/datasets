import cleanColumnNames from 'make_clean_names.js';

// Example usage
const data = [
  { Name: "John Doe", age: 30, EmailEmail_ID: "john@example.com" },
  { Name: "Jane Smith", age: 35, EmailEmail_ID: "jane@example.com" },
  { Name: "Bob Johnson", age: 40, EmailEmail_ID: "bob@example.com" }
];

const cleanedData = cleanColumnNames(data, {
  case: 'lower',
  separator: '_',
  stripAccents: true,
  insertUnderscores: true
});

console.log(cleanedData);