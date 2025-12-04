/*
 * Click nbfs://nbhost/SystemFileSystem/Templates/Licenses/license-default.txt to change this license
 * Click nbfs://nbhost/SystemFileSystem/Templates/Classes/Class.java to edit this template
 */
package ns;

/**
 *
 * @author angus
 */
public class CliRequest extends Request {

    public CliRequest(String[] args) {
        for (int i = 0; i < args.length; i++) {
            switch (args[i]) {
                case "--cli":
                    break;
                case "--nameserver":
                    if (i + 1 < args.length) {
                        this.nameserver = args[i + 1];
                        i++;
                    } else {
                        throw new IllegalArgumentException("Missing value for --nameserver");
                    }
                    break;
                case "--domain":
                case "-d":
                    if (i + 1 < args.length) {
                        this.domain = args[i + 1];
                        i++;
                    } else {
                        throw new IllegalArgumentException("Missing value for --domain");
                    }
                    break;
                case "--number":
                case "-n":
                    if (i + 1 < args.length) {
                        try {
                            this.number = Integer.parseInt(args[i + 1]);
                        } catch (NumberFormatException e) {
                            System.err.printf("Invalid number input. Must be integer. You entered %s\n", args[i + 1]);
                        }
                        i++;
                    } else {
                        throw new IllegalArgumentException("Missing value for --number");
                    }
                    break;
                case "--threads":
                case "-t":
                    if (i + 1 < args.length) {
                        try {
                            this.threads = Integer.parseInt(args[i + 1]);
                        } catch (NumberFormatException e) {
                            System.err.printf("Invalid threads input. Must be integer. You entered %s\n", args[i + 1]);
                        }
                        i++;
                    } else {
                        throw new IllegalArgumentException("Missing value for --threads");
                    }
                    break;
                case "--timeout":
                    if (i + 1 < args.length) {
                        try {
                            this.timeout = Integer.parseInt(args[i + 1]);
                        } catch (NumberFormatException e) {
                            System.err.printf("Invalid timeout input. Must be integer. You entered %s\n", args[i + 1]);
                            System.exit(1);
                        }
                        i++;
                    } else {
                        throw new IllegalArgumentException("Missing value for --timeout");
                    }
                    break;
                case "--verbose":
                case "-v":
                    this.verbose = true;
                    break;
                case "--random":
                    this.random = true;
                    break;
                case "--endless":
                    this.endless = true;
                    break;
                default:

                    throw new AssertionError();
            }
        }
    }

}
