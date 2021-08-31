#!/usr/bin/env python3

import json

from transformers import AutoTokenizer, AutoModelForTokenClassification

tokenizer = AutoTokenizer.from_pretrained("Jean-Baptiste/camembert-ner-with-dates")
model = AutoModelForTokenClassification.from_pretrained("Jean-Baptiste/camembert-ner-with-dates")

from transformers import pipeline

ner = pipeline('ner', model=model, tokenizer=tokenizer, grouped_entities=True)

def call(input):
    entities = ner(input)
    
    # converts digit to str (for json export)
    converted_entities = [{k: str(v) for (k,v) in i.items()} for i in entities]
    return json.dumps(converted_entities)
