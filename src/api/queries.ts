import {
  useMutation,
  useQuery,
  useQueryClient,
  type UseMutationResult,
} from "@tanstack/react-query"
import {
  assignStudentFee,
  generateInvoiceBatch,
  generateSupplementaryInvoice,
  getInvoiceItems,
  listClassInvoices,
  listInvoices,
  listStudentInvoices,
} from "./billing"
import {
  assignDiscount,
  createDiscount,
  listDiscounts,
  listStudentDiscounts,
} from "./discounts"
import {
  createClass,
  createTerm,
  deleteClass,
  deleteTerm,
  enrollStudent,
  getActiveTerm,
  listClasses,
  listEnrollments,
  listStudentEnrollments,
  listTerms,
  updateClass,
  updateTerm,
} from "./enrollment"
import { getFeeStructure, saveFeeStructure } from "./feeStructures"
import { listUsers } from "./auth"
import {
  getPayment,
  getPaymentAllocations,
  listPayments,
  listStudentPayments,
  recordPayment,
} from "./payments"
import { getAppInfo } from "./setup"
import { getInvoiceDetail, getStudentStatement } from "./statements"
import {
  getStudent,
  getStudentAccount,
  listStudents,
  listStudentsByStatus,
  registerStudent,
  updateStudentStatus,
} from "./students"
import type { AssignDiscountInput } from "../types/schema/AssignDiscountInput"
import type { AssignStudentFeeInput } from "../types/schema/AssignStudentFeeInput"
import type { Class } from "../types/schema/Class"
import type { CreateClassInput } from "../types/schema/CreateClassInput"
import type { CreateDiscountInput } from "../types/schema/CreateDiscountInput"
import type { CreateTermInput } from "../types/schema/CreateTermInput"
import type { Discount } from "../types/schema/Discount"
import type { EnrollStudentInput } from "../types/schema/EnrollStudentInput"
import type { Enrollment } from "../types/schema/Enrollment"
import type { FeeStructure } from "../types/schema/FeeStructure"
import type { GenerateBatchInput } from "../types/schema/GenerateBatchInput"
import type { GenerateSupplementaryInput } from "../types/schema/GenerateSupplementaryInput"
import type { Invoice } from "../types/schema/Invoice"
import type { Payment } from "../types/schema/Payment"
import type { RecordPaymentInput } from "../types/schema/RecordPaymentInput"
import type { RegisterStudentInput } from "../types/schema/RegisterStudentInput"
import type { SaveFeeStructureInput } from "../types/schema/SaveFeeStructureInput"
import type { Student } from "../types/schema/Student"
import type { StudentDiscount } from "../types/schema/StudentDiscount"
import type { StudentFee } from "../types/schema/StudentFee"
import type { Term } from "../types/schema/Term"
import type { UpdateClassInput } from "../types/schema/UpdateClassInput"
import type { UpdateStudentStatusInput } from "../types/schema/UpdateStudentStatusInput"
import type { UpdateTermInput } from "../types/schema/UpdateTermInput"

/* ---------------------------------------------------------------- queries */

export function useAppInfo() {
  return useQuery({ queryKey: ["app-info"], queryFn: getAppInfo })
}

export function useStudents(status?: string) {
  return useQuery({
    queryKey: ["students", status ?? "all"],
    queryFn: () =>
      status ? listStudentsByStatus(status) : listStudents(),
  })
}

export function useStudent(id: string | undefined) {
  return useQuery({
    queryKey: ["student", id],
    queryFn: () => getStudent(id as string),
    enabled: Boolean(id),
  })
}

export function useStudentAccount(studentId: string | undefined) {
  return useQuery({
    queryKey: ["student-account", studentId],
    queryFn: () => getStudentAccount(studentId as string),
    enabled: Boolean(studentId),
  })
}

export function useClasses() {
  return useQuery({ queryKey: ["classes"], queryFn: listClasses })
}

export function useTerms() {
  return useQuery({ queryKey: ["terms"], queryFn: listTerms })
}

export function useActiveTerm() {
  return useQuery({ queryKey: ["terms", "active"], queryFn: getActiveTerm })
}

export function useEnrollments(classId: string | undefined, termId: string | undefined) {
  return useQuery({
    queryKey: ["enrollments", classId, termId],
    queryFn: () => listEnrollments(classId as string, termId as string),
    enabled: Boolean(classId && termId),
  })
}

export function useStudentEnrollments(studentId: string | undefined) {
  return useQuery({
    queryKey: ["enrollments", "student", studentId],
    queryFn: () => listStudentEnrollments(studentId as string),
    enabled: Boolean(studentId),
  })
}

export function useFeeStructure(classId: string | undefined, termId: string | undefined) {
  return useQuery({
    queryKey: ["fee-structure", classId, termId],
    queryFn: () => getFeeStructure(classId as string, termId as string),
    enabled: Boolean(classId && termId),
  })
}

export function useDiscounts() {
  return useQuery({ queryKey: ["discounts"], queryFn: listDiscounts })
}

export function useStudentDiscounts(
  studentId: string | undefined,
  termId: string | undefined,
) {
  return useQuery({
    queryKey: ["student-discounts", studentId, termId],
    queryFn: () => listStudentDiscounts(studentId as string, termId as string),
    enabled: Boolean(studentId && termId),
  })
}

export function useInvoices() {
  return useQuery({ queryKey: ["invoices"], queryFn: listInvoices })
}

export function useStudentInvoices(studentId: string | undefined) {
  return useQuery({
    queryKey: ["invoices", "student", studentId],
    queryFn: () => listStudentInvoices(studentId as string),
    enabled: Boolean(studentId),
  })
}

export function useClassInvoices(classId: string | undefined, termId: string | undefined) {
  return useQuery({
    queryKey: ["invoices", "class", classId, termId],
    queryFn: () => listClassInvoices(classId as string, termId as string),
    enabled: Boolean(classId && termId),
  })
}

export function useInvoiceItems(invoiceId: string | undefined) {
  return useQuery({
    queryKey: ["invoice-items", invoiceId],
    queryFn: () => getInvoiceItems(invoiceId as string),
    enabled: Boolean(invoiceId),
  })
}

export function useInvoiceDetail(invoiceId: string | undefined) {
  return useQuery({
    queryKey: ["invoice-detail", invoiceId],
    queryFn: () => getInvoiceDetail(invoiceId as string),
    enabled: Boolean(invoiceId),
  })
}

export function usePayments() {
  return useQuery({ queryKey: ["payments"], queryFn: listPayments })
}

export function useUsers() {
  return useQuery({ queryKey: ["users"], queryFn: listUsers })
}

export function useStudentPayments(studentId: string | undefined) {
  return useQuery({
    queryKey: ["payments", "student", studentId],
    queryFn: () => listStudentPayments(studentId as string),
    enabled: Boolean(studentId),
  })
}

export function usePayment(id: string | undefined) {
  return useQuery({
    queryKey: ["payment", id],
    queryFn: () => getPayment(id as string),
    enabled: Boolean(id),
  })
}

export function usePaymentAllocations(paymentId: string | undefined) {
  return useQuery({
    queryKey: ["payment-allocations", paymentId],
    queryFn: () => getPaymentAllocations(paymentId as string),
    enabled: Boolean(paymentId),
  })
}

export function useStudentStatement(
  studentId: string | undefined,
  termId: string | null,
) {
  return useQuery({
    queryKey: ["statement", studentId, termId],
    queryFn: () => getStudentStatement(studentId as string, termId),
    enabled: Boolean(studentId),
  })
}

/* -------------------------------------------------------------- mutations */

/**
 * Invalidates every query that a write could have touched.
 *
 * The backend recalculates balances and statuses, so it is simpler and safer
 * to refetch broadly than to guess which slices changed.
 */
function useInvalidateAll() {
  const client = useQueryClient()
  return () => client.invalidateQueries()
}

export function useRegisterStudent(): UseMutationResult<
  Student,
  unknown,
  RegisterStudentInput
> {
  const invalidate = useInvalidateAll()
  return useMutation({
    mutationFn: registerStudent,
    onSuccess: invalidate,
  })
}

export function useUpdateStudentStatus(): UseMutationResult<
  void,
  unknown,
  UpdateStudentStatusInput
> {
  const invalidate = useInvalidateAll()
  return useMutation({
    mutationFn: updateStudentStatus,
    onSuccess: invalidate,
  })
}

export function useCreateClass(): UseMutationResult<
  Class,
  unknown,
  CreateClassInput
> {
  const invalidate = useInvalidateAll()
  return useMutation({ mutationFn: createClass, onSuccess: invalidate })
}

export function useCreateTerm(): UseMutationResult<
  Term,
  unknown,
  CreateTermInput
> {
  const invalidate = useInvalidateAll()
  return useMutation({ mutationFn: createTerm, onSuccess: invalidate })
}

export function useUpdateClass(): UseMutationResult<
  Class,
  unknown,
  UpdateClassInput
> {
  const invalidate = useInvalidateAll()
  return useMutation({ mutationFn: updateClass, onSuccess: invalidate })
}

export function useDeleteClass(): UseMutationResult<void, unknown, string> {
  const invalidate = useInvalidateAll()
  return useMutation({ mutationFn: deleteClass, onSuccess: invalidate })
}

export function useUpdateTerm(): UseMutationResult<
  Term,
  unknown,
  UpdateTermInput
> {
  const invalidate = useInvalidateAll()
  return useMutation({ mutationFn: updateTerm, onSuccess: invalidate })
}

export function useDeleteTerm(): UseMutationResult<void, unknown, string> {
  const invalidate = useInvalidateAll()
  return useMutation({ mutationFn: deleteTerm, onSuccess: invalidate })
}

export function useEnrollStudent(): UseMutationResult<
  Enrollment,
  unknown,
  EnrollStudentInput
> {
  const invalidate = useInvalidateAll()
  return useMutation({ mutationFn: enrollStudent, onSuccess: invalidate })
}

export function useSaveFeeStructure(): UseMutationResult<
  FeeStructure,
  unknown,
  SaveFeeStructureInput
> {
  const invalidate = useInvalidateAll()
  return useMutation({ mutationFn: saveFeeStructure, onSuccess: invalidate })
}

export function useCreateDiscount(): UseMutationResult<
  Discount,
  unknown,
  CreateDiscountInput
> {
  const invalidate = useInvalidateAll()
  return useMutation({ mutationFn: createDiscount, onSuccess: invalidate })
}

export function useAssignDiscount(): UseMutationResult<
  StudentDiscount,
  unknown,
  AssignDiscountInput
> {
  const invalidate = useInvalidateAll()
  return useMutation({ mutationFn: assignDiscount, onSuccess: invalidate })
}

export function useGenerateInvoiceBatch(): UseMutationResult<
  Invoice[],
  unknown,
  GenerateBatchInput
> {
  const invalidate = useInvalidateAll()
  return useMutation({ mutationFn: generateInvoiceBatch, onSuccess: invalidate })
}

export function useGenerateSupplementaryInvoice(): UseMutationResult<
  Invoice | null,
  unknown,
  GenerateSupplementaryInput
> {
  const invalidate = useInvalidateAll()
  return useMutation({
    mutationFn: generateSupplementaryInvoice,
    onSuccess: invalidate,
  })
}

export function useAssignStudentFee(): UseMutationResult<
  StudentFee,
  unknown,
  AssignStudentFeeInput
> {
  const invalidate = useInvalidateAll()
  return useMutation({ mutationFn: assignStudentFee, onSuccess: invalidate })
}

export function useRecordPayment(): UseMutationResult<
  Payment,
  unknown,
  RecordPaymentInput
> {
  const invalidate = useInvalidateAll()
  return useMutation({ mutationFn: recordPayment, onSuccess: invalidate })
}
