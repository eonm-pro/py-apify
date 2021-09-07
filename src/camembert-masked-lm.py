#!/usr/bin/env python3

import json

from transformers import CamembertModel, CamembertTokenizer

# You can replace "camembert-base" with any other model from the table, e.g. "camembert/camembert-large".
tokenizer = CamembertTokenizer.from_pretrained("camembert/camembert-large")
camembert = CamembertModel.from_pretrained("camembert/camembert-large")
from transformers import pipeline 

camembert_fill_mask  = pipeline("fill-mask", model="camembert/camembert-large", tokenizer="camembert/camembert-large", top_k = 10)

def call(input, top_k: int = 5):
    entities = camembert_fill_mask(input, top_k = top_k)
    
    # converts digit to str (for json export)
    converted_entities = [{k: str(v) for (k,v) in i.items()} for i in entities]
    return json.dumps(converted_entities)