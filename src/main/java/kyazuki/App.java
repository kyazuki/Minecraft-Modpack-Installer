package kyazuki;

import java.io.IOException;
import java.util.Locale;
import java.util.logging.FileHandler;
import java.util.logging.Handler;
import java.util.logging.LogManager;
import java.util.logging.Logger;
import java.util.logging.SimpleFormatter;

import javafx.application.Application;
import javafx.application.Platform;
import javafx.event.EventHandler;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.control.Alert;
import javafx.scene.control.Alert.AlertType;
import javafx.scene.control.DialogEvent;
import javafx.stage.Stage;
import kyazuki.dataclass.Config;

/**
 * JavaFX App
 */
public class App extends Application {
    /** ロガー */
    private static Logger logger = null;
    /** 設定 */
    private static Config config = null;
    /** ステージ */
    private static Stage stage = null;
    /** シーン */
    private static Scene scene = null;

    /**
     * JavaFX Appのエントリーポイント
     */
    @Override
    public void start(Stage stg) throws IOException {
        stage = stg;
        stage.setTitle("Minecraft Modpack Installer");
        scene = new Scene(loadFXML("configLoader"));
        stage.setScene(scene);
        stage.show();
    }

    /**
     * ロガーを取得する
     */
    public static Logger getLogger() {
        return logger;
    }

    /**
     * 設定を取得する
     */
    public static Config getConfig() {
        return config;
    }

    /**
     * 設定をセットする
     */
    public static void setConfig(Config cfg) {
        config = cfg;
    }

    /**
     * 指定されたFXMLをシーンにセットする
     * 
     * @param fxml FXMLファイル名
     * @throws IOException
     */
    public static void setRoot(String fxml) throws IOException {
        scene.setRoot(loadFXML(fxml));
    }

    /**
     * エラーアラートを表示する
     * 
     * @param message メッセージ
     */
    public static void showAlertAndExit(String message) {
        Alert alert = new Alert(AlertType.ERROR);
        alert.setTitle("エラー");
        alert.setHeaderText(null);
        alert.setContentText(message);
        alert.setOnCloseRequest(new EventHandler<DialogEvent>() {
            @Override
            public void handle(DialogEvent event) {
                Platform.exit();
            }
        });
        alert.show();
    }

    /**
     * 終了する
     */
    public static void finish() {
        stage.close();
    }

    /**
     * ロガーを初期化する
     */
    private static void initializeLogger() {
        LogManager.getLogManager().reset();
        Logger l = Logger.getLogger("minecraft-modpack-installer");
        Handler handler = null;
        try {
            handler = new FileHandler("installer.log");
            handler.setEncoding("UTF-8");
        } catch (SecurityException | IOException e) {
            e.printStackTrace();
            System.exit(1);
        }
        l.addHandler(handler);
        Locale.setDefault(new Locale("en", "EN"));
        System.setProperty("java.util.logging.SimpleFormatter.format",
                "%1$tY-%1$tm-%1$td %1$tH:%1$tM:%1$tS.%1$tL %4$s [%2$s] %5$s%6$s%n");
        handler.setFormatter(new SimpleFormatter());
        logger = l;
    }

    /**
     * FXMLをロードする
     * 
     * @param fxml FXMLファイル名
     * @return
     * @throws IOException
     */
    private static Parent loadFXML(String fxml) throws IOException {
        FXMLLoader fxmlLoader = new FXMLLoader(App.class.getResource(fxml + ".fxml"));
        return fxmlLoader.load();
    }

    public static void main(String[] args) {
        initializeLogger();
        launch();
    }

}