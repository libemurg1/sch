import { call } from "./client"
import type { FeeItem } from "../types/schema/FeeItem"
import type { FeeStructure } from "../types/schema/FeeStructure"
import type { SaveFeeStructureInput } from "../types/schema/SaveFeeStructureInput"

/**
 * The backend returns a Rust tuple, which serialises to a two element array:
 * `[structure, items]`. ts-rs cannot export tuples, so this type is written by
 * hand.
 */
export type FeeStructureBundle = [FeeStructure, FeeItem[]]

export function getFeeStructure(
  classId: string,
  termId: string,
): Promise<FeeStructureBundle | null> {
  return call<FeeStructureBundle | null>("get_fee_structure", { classId, termId })
}

/** Creates or replaces the fee structure for a class and term. */
export function saveFeeStructure(
  input: SaveFeeStructureInput,
): Promise<FeeStructure> {
  return call<FeeStructure>("save_fee_structure", { input })
}
