// List Ghidra references to selected addresses.
// Usage: -postScript ReferencesToAddresses.java 0x100da23c0 0x1000a861c

import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;
import ghidra.program.model.listing.Instruction;
import ghidra.program.model.symbol.Reference;
import ghidra.program.model.symbol.ReferenceIterator;

public class ReferencesToAddresses extends GhidraScript {
    @Override
    public void run() throws Exception {
        for (String argument : getScriptArgs()) {
            Address target = toAddr(argument);
            println("===== references to " + target + " =====");
            ReferenceIterator references = currentProgram
                .getReferenceManager()
                .getReferencesTo(target);
            while (references.hasNext()) {
                Reference reference = references.next();
                Address source = reference.getFromAddress();
                Function function = getFunctionContaining(source);
                Instruction instruction = getInstructionAt(source);
                String functionName = function == null
                    ? "<data>"
                    : function.getName() + " @ " + function.getEntryPoint();
                println(
                    source + " " + reference.getReferenceType() + " " + functionName
                        + (instruction == null ? "" : " :: " + instruction)
                );
            }
        }
    }
}
