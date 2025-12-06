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

    public static final String RED     = "\033[31m";
    public static final String GREEN   = "\033[32m";
    public static final String YELLOW  = "\033[33m";
    public static final String BLUE    = "\033[34m";
    public static final String MAGENTA = "\033[35m";
    public static final String CYAN    = "\033[36m";
    public static final String RESET   = "\033[0m";
    public static final String BOLD    = "\033[1m";
    public static final String ORANGE  = "\033[38;5;208m";

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
        if (nameserver == null) {
            System.out.println(RED + "No Nameserver selected. You must select a nameserver." + RESET);
            System.exit(1);
        }

        print_options();
    }

    private void print_options() {
        String asciiBanner = String.format(
            """
            %s%s
                        _   _  _____       _____ _____        __  __ 
                        | \\ | |/ ____|     / ____|  __ \\ /\\   |  \\/  |
                        |  \\| | (___ _____| (___ | |__) /  \\  | \\  / |
                        | . ` |\\___ \\______\\___ \\|  ___/ /\\ \\ | |\\/| |
                        | |\\  |____) |     ____) | |  / ____ \\| |  | |
                        |_| \\_|_____/     |_____/|_| /_/    \\_\\_|  |_|                                   
            %s
            """,
            CYAN, BOLD, RESET
        );

        // Build message
        StringBuilder sb = new StringBuilder();

        sb.append(String.format("%sThank you for using %sNS-Spam%s.\n",
                YELLOW, MAGENTA, RESET));

        sb.append(String.format(
                "%sThis application should only be run on nameservers you have permission from the owner to use.%s\n",
                YELLOW, RESET));

        sb.append(String.format(
                "%sYou have selected nameserver %s%s%s %s to run on.%s\n",
                YELLOW, BOLD, BLUE, nameserver, RESET, YELLOW, RESET));

        if (random) {
            sb.append(String.format(
                    "%sRandom option selected.%s %sThis will create random subdomains off of your domain %s%s%s %s with a length of 3-7 characters.%s\n",
                    YELLOW, RESET, YELLOW, BOLD, BLUE, domain, RESET, YELLOW, RESET
            ));
        } else {
            sb.append(String.format(
                    "%sYou have selected the domain %s%s%s %s to run on.%s\n",
                    YELLOW, BOLD, BLUE, domain, RESET, YELLOW, RESET
            ));
        }

        System.out.println(asciiBanner);
        System.out.println(MAGENTA + "-".repeat(114) + RESET);
        System.out.println(sb);
        System.out.println(MAGENTA + "-".repeat(114) + RESET);
        System.out.println("\n" + YELLOW + "Press enter to begin or CTRL+C to exit" + RESET);

        Thread shutdownHook = new Thread(() -> {
            System.out.println("\n" + GREEN + "Exiting gracefully (Ctrl+C detected)" + RESET);
        });
        Runtime.getRuntime().addShutdownHook(shutdownHook);

        try {
            System.in.read();
        } catch (Exception e) {
            System.out.println("\n" + GREEN + "Exiting gracefully" + RESET);
            System.exit(0);
        } finally {
            try {
                Runtime.getRuntime().removeShutdownHook(shutdownHook);
            } catch (IllegalStateException ignored) {
            }
        }
    }

}
