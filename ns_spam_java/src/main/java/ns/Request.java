/*
 * Click nbfs://nbhost/SystemFileSystem/Templates/Licenses/license-default.txt to change this license
 * Click nbfs://nbhost/SystemFileSystem/Templates/Classes/Class.java to edit this template
 */
package ns;

import org.xbill.DNS.*;
import java.net.InetAddress;
import java.util.ArrayList;
import java.util.List;

/**
 *
 * @author angus
 */
public abstract class Request {

    protected String nameserver;
    protected String domain = "google.com";
    protected int number = 100;
    protected int threads = 10;
    protected int timeout = 5;
    protected boolean verbose = false;
    protected boolean random = false;
    protected boolean endless = false;

    public int run_request() {
        System.out.println(number);
        for (int i = 0; i < number; i++) {
            make_request();
        }
        return 0;
    }

    protected int make_request() {
        try {
            // Create a SimpleResolver and set the nameserver
            SimpleResolver resolver = new SimpleResolver(nameserver);

            // Create a Record for the query
            Name name = Name.fromString(domain + ".");
            org.xbill.DNS.Record question = org.xbill.DNS.Record.newRecord(name, Type.A, DClass.IN);

            // Create a Message and add the question
            Message query = Message.newQuery(question);

            // Send the query and get the response
            Message response = resolver.send(query);

            // Parse the response to get the A records (IP addresses)
            org.xbill.DNS.Record[] answers = response.getSectionArray(Section.ANSWER);
            List<InetAddress> ipAddresses = new ArrayList<>();

            for (org.xbill.DNS.Record r : answers) {
                if (r instanceof org.xbill.DNS.ARecord) {
                    org.xbill.DNS.ARecord a = (org.xbill.DNS.ARecord) r;
                    ipAddresses.add(a.getAddress());
                }
            }

            if (!ipAddresses.isEmpty()) {
                System.out.println("IP addresses for " + domain + " using nameserver " + nameserver + ":");
                for (InetAddress ip : ipAddresses) {
                    System.out.println(ip.getHostAddress());
                }
            } else {
                System.out.println("No IP addresses found for " + domain + " using nameserver " + nameserver);
            }

        } catch (Exception e) {
            System.err.println("DNS lookup failed: " + e.getMessage());
        }

        return 0;
    }
}
