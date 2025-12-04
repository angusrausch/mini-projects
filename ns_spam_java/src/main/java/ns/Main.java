package ns;

public class Main {
    public static void main(String[] args) {
        if (args.length > 0) {
            if (args[0].equals("--cli")) {
                CliRequest cli = new CliRequest(args);
                cli.run_request();
            }
        }
        
    }
}