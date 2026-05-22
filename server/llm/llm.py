import gc
import logging as log
from typing import Any

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig


class Llm:
    def __init__(self, model_name: str = "Qwen/Qwen2.5-7B-Instruct", device_map: Any = None) -> None:
        self.dbg = "Llm"

        self.model_name = model_name
        self.device_map = device_map or {
            "model.embed_tokens": 0,
            "model.layers.0": 0,
            "model.layers.1": 0,
            "model.layers.2": 0,
            "model.layers.3": 0,
            "model.layers.4": 0,
            "model.layers.5": 0,
            "model.layers.6": 0,
            "model.layers.7": 0,
            "model.layers.8": 0,
            "model.layers.9": 0,
            "model.layers.10": 0,
            "model.layers.11": 0,
            "model.layers.12": 0,
            "model.layers.13": 0,
            "model.layers.14": 0,
            "model.layers.15": 0,
            "model.layers.16": 1,
            "model.layers.17": 1,
            "model.layers.18": 1,
            "model.layers.19": 1,
            "model.layers.20": 1,
            "model.layers.21": 1,
            "model.layers.22": 1,
            "model.layers.23": 1,
            "model.layers.24": 1,
            "model.layers.25": 1,
            "model.layers.26": 1,
            "model.layers.27": 1,
            "model.norm": 1,
            "lm_head": 0,
        }
        self.bnb_config = BitsAndBytesConfig(
            load_in_4bit=True,
            bnb_4bit_compute_dtype=torch.float16,
            bnb_4bit_quant_type="nf4",
            bnb_4bit_use_double_quant=True,
        )
        self.model = None
        self.tokenizer = None

    def _build_prompt(self, query: str) -> str:
        return (
            "Ты консультант который отвечает на вопросы пользователей. "
            "Отвечай ТОЛЬКО на основе предоставленного текста. "
            "Если информации недостаточно ответь - я не знаю.\n\n"
            f"Вопрос: {query}\n"
            "Ответ:"
        )

    def run(self) -> None:
        log.debug(f'{self.dbg}.run | Loading model {self.model_name}')
        torch.cuda.empty_cache()
        torch.cuda.reset_peak_memory_stats()

        self.model = AutoModelForCausalLM.from_pretrained(
            self.model_name,
            dtype=torch.float16,
            device_map=self.device_map,
            low_cpu_mem_usage=True,
            # quantization_config=self.bnb_config,
        )
        self.tokenizer = AutoTokenizer.from_pretrained(self.model_name)
        self.model.eval()

        gc.collect()
        log.debug(f'{self.dbg}.run | Model loaded')

    def handleQueries(self, query: str) -> str:
        log.debug(f'{self.dbg}.handleQueries | Query: {query}')
        if self.model is None or self.tokenizer is None:
            raise RuntimeError("LLM is not initialized. Call run() before handleQueries().")

        prompt = self._build_prompt(query)
        device = "cuda:0" if torch.cuda.is_available() else "cpu"
        inputs = self.tokenizer(prompt, return_tensors="pt")
        inputs = {key: tensor.to(device) for key, tensor in inputs.items()}

        with torch.no_grad():
            generated_ids = self.model.generate(
                **inputs,
                max_new_tokens=128,
                do_sample=True,
                temperature=0.7,
                top_p=0.9,
                top_k=50,
                repetition_penalty=1.1,
                pad_token_id=self.tokenizer.pad_token_id or self.tokenizer.eos_token_id,
                eos_token_id=self.tokenizer.eos_token_id,
            )

        reply = self.tokenizer.decode(generated_ids[0], skip_special_tokens=True)
        log.debug(f'{self.dbg}.handleQueries | Reply: {reply}')
        return reply
