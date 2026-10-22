import { commands as native } from "./generated/bindings";
import { write } from "./write";

export const terminologyCommands = {
  appendTermbase(
    access: Parameters<typeof native.appendTermbase>[0],
    termbaseId: Parameters<typeof native.appendTermbase>[1],
    document: Parameters<typeof native.appendTermbase>[2],
  ): ReturnType<typeof native.appendTermbase> {
    return write("appendTermbase", [access, termbaseId, document], (request) =>
      native.appendTermbase(access, termbaseId, document, request),
    );
  },
  importTermbase(
    access: Parameters<typeof native.importTermbase>[0],
    document: Parameters<typeof native.importTermbase>[1],
  ): ReturnType<typeof native.importTermbase> {
    return write("importTermbase", [access, document], (request) =>
      native.importTermbase(access, document, request),
    );
  },
  createTermbase(
    access: Parameters<typeof native.createTermbase>[0],
    metadata: Parameters<typeof native.createTermbase>[1],
  ): ReturnType<typeof native.createTermbase> {
    return write("createTermbase", [access, metadata], (request) =>
      native.createTermbase(access, metadata, request),
    );
  },
  updateTermbase(
    access: Parameters<typeof native.updateTermbase>[0],
    baseline: Parameters<typeof native.updateTermbase>[1],
    metadata: Parameters<typeof native.updateTermbase>[2],
  ): ReturnType<typeof native.updateTermbase> {
    return write("updateTermbase", [access, baseline, metadata], (request) =>
      native.updateTermbase(access, baseline, metadata, request),
    );
  },
  deleteTermbase(
    access: Parameters<typeof native.deleteTermbase>[0],
    baseline: Parameters<typeof native.deleteTermbase>[1],
  ): ReturnType<typeof native.deleteTermbase> {
    return write("deleteTermbase", [access, baseline], (request) =>
      native.deleteTermbase(access, baseline, request),
    );
  },
  createTerm(
    access: Parameters<typeof native.createTerm>[0],
    termbaseId: Parameters<typeof native.createTerm>[1],
    metadata: Parameters<typeof native.createTerm>[2],
  ): ReturnType<typeof native.createTerm> {
    return write("createTerm", [access, termbaseId, metadata], (request) =>
      native.createTerm(access, termbaseId, metadata, request),
    );
  },
  updateTerm(
    access: Parameters<typeof native.updateTerm>[0],
    baseline: Parameters<typeof native.updateTerm>[1],
    metadata: Parameters<typeof native.updateTerm>[2],
  ): ReturnType<typeof native.updateTerm> {
    return write("updateTerm", [access, baseline, metadata], (request) =>
      native.updateTerm(access, baseline, metadata, request),
    );
  },
  deleteTerm(
    access: Parameters<typeof native.deleteTerm>[0],
    baseline: Parameters<typeof native.deleteTerm>[1],
  ): ReturnType<typeof native.deleteTerm> {
    return write("deleteTerm", [access, baseline], (request) =>
      native.deleteTerm(access, baseline, request),
    );
  },
};
