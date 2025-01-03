// Fallback for unidecode if not available
const unidecode = (str) => {
  // Basic accent removal implementation
  return str.normalize('NFD')
            .replace(/[\u0300-\u036f]/g, '')
            .replace(/[^\w\s]/g, '');
};

/**
 * Clean and standardize column names
 * @param {Object[] | any} data - The input data 
 * @param {Object} options - Configuration options for cleaning
 * @returns {Object[]} - Transformed data with cleaned column names
 */
export function cleanColumnNames(data, options = {}) {
  // Default options
  const defaultOptions = {
    case: 'lower', // 'lower', 'upper', or 'proper'
    separator: '_',
    stripAccents: true,
    insertUnderscores: true,
    truncateLimit: null,
    skipPatterns: [],
    preserveSpecialWords: {}
  };

  // Merge default options with provided options
  const config = { ...defaultOptions, ...options };

  // Validation and type checking
  if (data == null) {
    console.warn('cleanColumnNames: Input data is null or undefined');
    return [];
  }

  // Ensure data is an array of objects
  let processedData = Array.isArray(data) ? data : [data];

  // Helper function to clean a single name
  const cleanName = (name) => {
    // Handle null or undefined
    if (name == null) return '';
    
    // Convert to string
    let cleanedName = String(name);

    // Strip accents if configured
    if (config.stripAccents) {
      try {
        cleanedName = unidecode(cleanedName);
      } catch (error) {
        console.warn('Error stripping accents:', error);
      }
    }

    // Remove special characters and replace with separator
    cleanedName = cleanedName
      .replace(/[^\w\s]/g, config.separator)
      .replace(/\s+/g, config.separator);

    // Insert underscores between camelCase
    if (config.insertUnderscores) {
      cleanedName = cleanedName.replace(/([a-z0-9])([A-Z])/g, `$1${config.separator}$2`);
    }

    // Remove leading/trailing separators
    cleanedName = cleanedName.replace(new RegExp(`^${config.separator}+|${config.separator}+$`, 'g'), '');

    // Reduce multiple consecutive separators
    cleanedName = cleanedName.replace(new RegExp(`${config.separator}+`, 'g'), config.separator);

    // Apply case transformation
    switch (config.case) {
      case 'lower':
        cleanedName = cleanedName.toLowerCase();
        break;
      case 'upper':
        cleanedName = cleanedName.toUpperCase();
        break;
      case 'proper':
        cleanedName = cleanedName
          .split(config.separator)
          .map((word, index) => 
            // Capitalize first letter, keep rest lowercase
            word.charAt(0).toUpperCase() + word.slice(1).toLowerCase()
          )
          .join(config.separator);
        break;
    }

    // Truncate if limit specified
    if (config.truncateLimit) {
      cleanedName = cleanedName.slice(0, config.truncateLimit);
    }

    // Skip cleaning for specified patterns
    if (config.skipPatterns.some(pattern => 
      new RegExp(pattern, 'i').test(cleanedName)
    )) {
      return name;
    }

    return cleanedName;
  };

  // If no data or empty array, return empty array
  if (processedData.length === 0) {
    return [];
  }

  // Determine if we're dealing with an array of objects or a single object
  if (typeof processedData[0] !== 'object') {
    console.warn('cleanColumnNames: Input is not an array of objects');
    return processedData;
  }

  // Clean column names
  const cleanedData = processedData.map(row => {
    if (row == null) return null;
    
    const cleanedRow = {};
    
    // Handle both object and array-like structures
    const keys = Object.keys(row);
    
    keys.forEach(key => {
      const newKey = cleanName(key);
      cleanedRow[newKey] = row[key];
    });
    
    return cleanedRow;
  });

  return cleanedData;
}

// Fallback transformation for various data types
export function transformData(data, options = {}) {
  try {
    // If data is a DataFrame or similar, extract underlying data
    if (data && typeof data.toArray === 'function') {
      data = data.toArray();
    }
    
    // If data is a CSV string, parse it
    if (typeof data === 'string') {
      if (typeof Papa !== 'undefined') {
        data = Papa.parse(data, { header: true }).data;
      } else {
        console.error('Papa Parse library not available for CSV parsing');
        return [];
      }
    }
    
    return cleanColumnNames(data, options);
  } catch (error) {
    console.error('Error transforming data:', error);
    return [];
  }
}

export default cleanColumnNames;