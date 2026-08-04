// Decompile selected function addresses from a headless Ghidra project.
// Usage: -postScript DecompileAddresses.java 0x1001920dc 0x10019216a

import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;

public class DecompileAddresses extends GhidraScript {
    @Override
    public void run() throws Exception {
        DecompInterface decompiler = new DecompInterface();
        decompiler.toggleCCode(true);
        decompiler.toggleSyntaxTree(true);
        if (!decompiler.openProgram(currentProgram)) {
            throw new IllegalStateException("Could not open the current program in the decompiler");
        }

        for (String argument : getScriptArgs()) {
            Address address = toAddr(argument);
            Function function = getFunctionContaining(address);
            if (function == null) {
                println("===== " + argument + ": no containing function =====");
                continue;
            }

            println(
                "===== " + argument + " " + function.getName() + " @ "
                    + function.getEntryPoint() + " ====="
            );
            DecompileResults result = decompiler.decompileFunction(function, 120, monitor);
            if (!result.decompileCompleted() || result.getDecompiledFunction() == null) {
                println("Decompiler failed: " + result.getErrorMessage());
                continue;
            }
            println(result.getDecompiledFunction().getC());
        }
        decompiler.dispose();
    }
}
