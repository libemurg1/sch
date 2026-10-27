import { call } from "./client"
import type { ApplyPromotionInput } from "../types/schema/ApplyPromotionInput"
import type { PromotionPreview } from "../types/schema/PromotionPreview"
import type { PromotionResult } from "../types/schema/PromotionResult"

/** Creates the missing classes of `toYear` by copying `fromYear`. */
export function copyClasses(fromYear: number, toYear: number): Promise<number> {
  return call<number>("copy_classes", { fromYear, toYear })
}

export function previewPromotion(
  fromYear: number,
  toYear: number,
): Promise<PromotionPreview> {
  return call<PromotionPreview>("preview_promotion", { fromYear, toYear })
}

export function applyPromotion(
  input: ApplyPromotionInput,
): Promise<PromotionResult> {
  return call<PromotionResult>("apply_promotion", { input })
}
