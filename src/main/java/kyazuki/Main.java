package kyazuki;

/**
 * Jarファイルから実行できるようにするためのクラス
 * https://stackoverflow.com/questions/52653836/maven-shade-javafx-runtime-components-are-missing
 */
public class Main {
    public static void main(String[] args) {
        App.main(args);
    }
}
