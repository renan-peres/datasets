import polars as pl
import re
from typing import Union, Optional, List, Dict
from unidecode import unidecode

def make_clean_names(
    df: Union[pl.DataFrame, pl.LazyFrame],
    axis: str = "columns",
    column_names: Union[str, List[str]] = None,
    strip_underscores: Optional[Union[str, bool]] = None,
    case_type: str = "lower",
    replace_character: Dict[str, str] = None,
    strip_accents: bool = True,
    truncate_limit: int = None,
    insert_underscores: bool = True,
    capitalize_first_letter: bool = False,
    skip_patterns: List[str] = None
) -> Union[pl.DataFrame, pl.LazyFrame]:
    """
    Clean and standardize column names for Polars DataFrames.
    Uses underscores as default word separator unless explicitly specified to use spaces.

    Parameters:
    df (Union[pl.DataFrame, pl.LazyFrame]): The DataFrame whose names need to be cleaned
    axis (str): The axis to clean names for. Currently only supports "columns"
    column_names (Union[str, List[str], None]): Specific column names to clean. If None, all columns are cleaned
    strip_underscores (Optional[Union[str, bool]]): How to strip underscores. Options: "left", "right", "both", or True
    case_type (str): Case type to convert names to. Options: "lower", "upper", "proper"
    replace_character (Dict[str, str]): Dictionary mapping strings to replacements. Add '_': ' ' to use spaces
    strip_accents (bool): Whether to strip accents from names
    truncate_limit (int): If provided, truncate names to this limit
    insert_underscores (bool): Whether to insert underscores between words
    capitalize_first_letter (bool): Whether to capitalize first letter of each word in proper case
    skip_patterns (List[str]): List of patterns to skip

    Returns:
    Union[pl.DataFrame, pl.LazyFrame]: DataFrame with cleaned column names
    """
    def smart_replace(name: str, replacements: Dict[str, str]) -> str:
        """Performs replacements at string start, word boundaries, and string end."""
        if not name:
            return ""
            
        use_spaces = '_' in replacements and replacements['_'] == ' '
        
        # Standardize all separators to underscores
        name = re.sub(r'\s+', '_', name)
        
        # Sort replacements by length (longest first) to avoid partial matches
        sorted_items = sorted(replacements.items(), key=lambda x: len(x[0]), reverse=True)
        
        # Split into words
        words = name.split('_')
        
        # Process each word
        for i in range(len(words)):
            word = words[i]
            word_lower = word.lower()
            for old, new in sorted_items:
                if old != '_' and word_lower == old.lower():
                    words[i] = new
                    break
        
        # Join words back together
        separator = ' ' if use_spaces else '_'
        return separator.join(words)

    def clean_name(name: str) -> str:
        """Clean and standardize a name."""
        if name is None:
            return ""
            
        if skip_patterns and any(pattern in str(name).lower() for pattern in skip_patterns):
            return str(name)

        name = str(name)
        
        if strip_accents:
            name = unidecode(name)
        
        # Apply replacements
        if replace_character:
            name = smart_replace(name, replace_character)
        
        # Determine separator type
        use_spaces = replace_character and '_' in replace_character and replace_character['_'] == ' '
        separator = ' ' if use_spaces else '_'
        
        # Standardize spaces to separator
        name = re.sub(r'\s+', separator, name)
        
        if strip_underscores:
            name = name.strip(separator) if strip_underscores == "both" else (
                name.lstrip(separator) if strip_underscores == "left" else (
                    name.rstrip(separator) if strip_underscores == "right" else name.replace(separator, separator)
                )
            )
        
        if insert_underscores:
            name = re.sub(r'([a-z])([A-Z])', fr'\1{separator}\2', name)
        
        # Clean up the name
        name = re.sub(r'(^|\s|_)([a-zA-Z])-([a-zA-Z]+)(\s|$|_)', r'\1\2\3\4', name)
        name = re.sub(r'\W+', separator, name)
        name = re.sub(f'^[{separator}\\W]+|[{separator}\\W]+$', '', name)
        name = re.sub(f'[{separator}]{{2,}}', separator, name)
        name = re.sub(r'([a-zA-Z])(\d+)$', fr'\1{separator}\2', name)

        # Process case
        words = name.split(separator)
        case_type_lower = case_type.lower()
        
        if case_type_lower == "lower":
            words = [w.lower() for w in words]
        elif case_type_lower == "upper":
            words = [w.upper() for w in words]
        elif case_type_lower == "proper":
            words = [w.capitalize() if capitalize_first_letter or i == 0 else w.lower() 
                    for i, w in enumerate(words)]
        
        name = separator.join(words)
        
        if truncate_limit:
            name = name[:truncate_limit]
        
        # Final cleanup
        name = re.sub(f'(^|{separator})([a-z]){separator}([a-z])({separator}|$)', r'\1\2\3\4', name)
        name = re.sub(f'{separator}([a-zA-Z]){separator}(\\d+)$', r'\1\2', name)
        
        return name

    if axis == "columns":
        try:
            original_columns = df.columns
            if column_names:
                column_names = [column_names] if isinstance(column_names, str) else column_names
                new_columns = [clean_name(col) if col in column_names else col for col in original_columns]
            else:
                new_columns = [clean_name(col) for col in original_columns]
            
            df = df.rename({original: new for original, new in zip(original_columns, new_columns)})
        except Exception as e:
            print(f"Error processing DataFrame: {str(e)}")

    return df