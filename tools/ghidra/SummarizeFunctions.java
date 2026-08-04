// Summarize selected functions and their direct call graph.

import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;

public class SummarizeFunctions extends GhidraScript {
    @Override
    public void run() throws Exception {
        for (String argument : getScriptArgs()) {
            Address address = toAddr(argument);
            Function function = getFunctionContaining(address);
            if (function == null) {
                println(argument + " <no function>");
                continue;
            }
            println(
                "===== " + argument + " " + function.getName() + " @ "
                    + function.getEntryPoint() + " size="
                    + function.getBody().getNumAddresses() + " ====="
            );
            println("  callers:");
            for (Function caller : function.getCallingFunctions(monitor)) {
                println("    " + caller.getEntryPoint() + " " + caller.getName());
            }
            println("  callees:");
            for (Function callee : function.getCalledFunctions(monitor)) {
                println("    " + callee.getEntryPoint() + " " + callee.getName());
            }
        }
    }
}
