default: help

.PHONY: demo_object_injection
demo_object_injection: target/debug/ism ## Demo of Object Injection (Listing 1/12)
	$(call run_demo,listing-1_object-injection)

.PHONY: demo_prototype_pollution
demo_prototype_pollution: target/debug/ism ## Demo of Prototype Pollution (Listing 2/13)
	$(call run_demo,listing-2_prototype-pollution)

.PHONY: demo_class_pollution
demo_class_pollution: target/debug/ism ## Demo of Class Pollution (Listing 3/14)
	$(call run_demo,listing-3_class-pollution)

.PHONY: demo_dom_clobbering
demo_dom_clobbering: target/debug/ism ## Demo of DOM Clobbering (Listing 4/15)
	$(call run_demo,listing-4_dom-clobbering)

.PHONY: demo_dom_script_gadget
demo_dom_script_gadget: target/debug/ism ## Demo of DOM Script Gadgets (Listing 5/16)
	$(call run_demo,listing-5_dom-script-gadget)

.PHONY: demo_weakness_recycle
demo_weakness_recycle: target/debug/ism ## Demo of a recycling weakness (Listing 6)
	$(call run_demo,listing-6_recycle)

.PHONY: demo_weakness_stateless_prototype
demo_weakness_stateless_prototype: target/debug/ism ## Demo of a stateless prototype-based weakness (Listing 7)
	$(call run_demo,listing-7_stateless-prototype)

.PHONY: demo_weakness_mutate_fixed
demo_weakness_mutate_fixed: target/debug/ism ## Demo of a mutating weakness using a fixed value (Listing 8)
	$(call run_demo,listing-8_mutate-fixed)

.PHONY: demo_gadget_vertical_chain
demo_gadget_vertical_chain: target/debug/ism ## Demo of a vertical gadget chain (Listing 9)
	$(call run_demo,listing-9_vertical-chain)

.PHONY: demo_gadget_horizontal_chain
demo_gadget_horizontal_chain: target/debug/ism ## Demo of a horizontal gadget chain (Listing 10)
	$(call run_demo,listing-10_horizontal-chain)

.PHONY: demo_gadget_horizontal_chain_data
demo_gadget_horizontal_chain_data: target/debug/ism ## Demo of a data dependency horizontal gadget chain (Listing 11)
	$(call run_demo,listing-11_horizontal-chain-data)

.PHONY: demo_definition_2
demo_definition_2: target/debug/ism ## Demo of a false positive for Definition 2 (Listing 17)
	$(call run_demo,listing-17_definition-2)

.PHONY: demo_definition_3
demo_definition_3: target/debug/ism ## Demo of a false positive for Definition 3 (Listing 18)
	$(call run_demo,listing-18_definition-3)

.PHONY: demo_definition_stateful
demo_definition_stateful: target/debug/ism ## Demo of a false positive for stateful security condition (Listing 19)
	$(call run_demo,listing-19_definition-stateful)

.PHONY: demo_observation_order
demo_observation_order: target/debug/ism ## Demo of an attacker controlling the observation order (Listing 20)
	$(call run_demo,listing-20_observation-order)

.PHONY: demo_weakness_categorization
demo_weakness_categorization: target/debug/ism ## Demo of weakness categorization examples (Figure 8)
	$(call run_demo,figure-8-01_stateless-u)
	$(call run_demo,figure-8-03_mutate-uuu)
	$(call run_demo,figure-8-04_mutate-uut)
	$(call run_demo,figure-8-05_mutate-utu)
	$(call run_demo,figure-8-06_mutate-utt)
	$(call run_demo,figure-8-07_mutate-tuu)
	$(call run_demo,figure-8-09_mutate-ttu)
	$(call run_demo,figure-8-11_recycle-uuu)
	$(call run_demo,figure-8-12_recycle-uut)
	$(call run_demo,figure-8-13_recycle-utu)
	$(call run_demo,figure-8-14_recycle-utt)
	$(call run_demo,figure-8-15_recycle-tuu)
	$(call run_demo,figure-8-16_recycle-tut)
	$(call run_demo,figure-8-17_recycle-ttu)

.PHONY: demo_gadget_categorization
demo_gadget_categorization: target/debug/ism ## Demo of gadget categorization examples (Table 3)
	$(call run_demo,table-3-01_sink)
	$(call run_demo,table-3-02_call-sink)
	$(call run_demo,table-3-03_control-sink)
	$(call run_demo,table-3-04_control-call-sink)
	$(call run_demo,table-3-05_control-data-sink)
	$(call run_demo,table-3-06_control-data-call-sink)
	$(call run_demo,table-3-07_control-data-call-data-sink)
	$(call run_demo,table-3-08_control-call-data-sink)
	$(call run_demo,table-3-09_control-call-data-call-sink)
	$(call run_demo,table-3-10_call-data-sink)
	$(call run_demo,table-3-11_call-data-control-sink)
	$(call run_demo,table-3-12_call-data-control-data-sink)
	$(call run_demo,table-3-13_call-data-control-call-sink)
	$(call run_demo,table-3-14_fetch-data-sink)
	$(call run_demo,table-3-15_call-fetch-data-sink)
	$(call run_demo,table-3-16_control-fetch-data-sink)
	$(call run_demo,table-3-17_control-call-fetch-data-sink)

.PHONY: web
web: wasm ## Build and serve the web-based interpreter
	@python3 -m http.server

.PHONY: cli wasm
cli: target/debug/ism
wasm: pkg/ism_bg.wasm

.PHONY: clean
clean: ## Clean the project
	@cargo clean
	@rm -rf pkg/
	@rm -f *.mermaid

.PHONY: help
help:
	@printf "Usage: make <command>\n\n"
	@printf "Commands:\n"
	@awk -F ':(.*)## ' '/^[a-zA-Z0-9%\\\/_.-]+:(.*)##/ { \
		printf "  \033[36m%-34s\033[0m %s\n", $$1, $$NF \
	}' $(MAKEFILE_LIST)

define run_demo
	@cat inputs/$(1).1 inputs/$(1).2 | ./target/debug/ism compare programs/$(1).ism
endef

target/debug/ism: Cargo.lock Cargo.toml src/lib.rs src/main.rs
	@cargo build
	@echo #newline

pkg/ism_bg.wasm: Cargo.lock Cargo.toml src/lib.rs
ifneq (, $(shell which wasm-pack))
	@wasm-pack build --target web --no-default-features --features wasm
	@echo #newline
else
	@echo 'This command requires wasm pack (run "cargo install wasm-pack").'
	@exit 1
endif
