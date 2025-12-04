mvn compile

echo "\n\n==============================================\n\n"

mvn exec:java -Dexec.mainClass=ns.Main -Dexec.args="$*"
